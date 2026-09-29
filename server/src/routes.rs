use std::sync::{Arc, Mutex};

use axum::Router;
use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use tfhe::{CompressedServerKey, FheUint32, ServerKey};

use blindtally_core::election::Ballot;
use blindtally_core::errors::AppError;
use blindtally_core::io;

use crate::error::ApiError;
use crate::tally;

/// Everything the server knows about the election. Each field is either
/// public (the server key) or encrypted (ballots, counts).
#[derive(Default)]
pub struct Election {
    server_key: Option<ServerKey>,
    ballots: Vec<Ballot>,
    counts: Option<Vec<FheUint32>>,
}

/// Handlers run concurrently on many threads, so the election is shared
/// through an `Arc` (shared ownership) and guarded by a `Mutex` (one
/// accessor at a time). A std `MutexGuard` must never be held across an
/// `.await`; the compiler enforces this because axum needs `Send` futures.
type AppState = Arc<Mutex<Election>>;

/// Even compressed, the server key is far above axum's 2 MB default limit.
const MAX_KEY_BYTES: usize = 256 * 1024 * 1024;

pub fn router() -> Router {
    Router::new()
        .route(
            "/keys",
            post(upload_key).layer(DefaultBodyLimit::max(MAX_KEY_BYTES)),
        )
        .route("/ballots", post(cast_ballot))
        .route("/tally", post(run_tally))
        .route("/results", get(results))
        .with_state(AppState::default())
}

fn not_open() -> ApiError {
    ApiError::new(
        StatusCode::CONFLICT,
        "election not open: upload a server key first",
    )
}

/// `POST /keys` — body: bincode `CompressedServerKey`. Opens the election.
async fn upload_key(State(state): State<AppState>, body: Bytes) -> Result<StatusCode, ApiError> {
    // Decoding and decompressing take seconds of CPU. Doing that directly in
    // an async fn would stall every other request on this worker thread, so
    // it runs on tokio's dedicated pool for blocking work instead.
    let server_key = tokio::task::spawn_blocking(move || -> Result<ServerKey, AppError> {
        let compressed: CompressedServerKey = io::from_bytes(&body)?;
        Ok(compressed.decompress())
    })
    .await??;

    let mut election = state.lock().unwrap();
    // Ballots are only meaningful under the key they were encrypted for,
    // so the key cannot be swapped once set.
    if election.server_key.is_some() {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            "election already has a server key",
        ));
    }
    election.server_key = Some(server_key);
    tracing::info!("election opened");
    Ok(StatusCode::CREATED)
}

/// `POST /ballots` — body: bincode `Ballot`.
async fn cast_ballot(State(state): State<AppState>, body: Bytes) -> Result<StatusCode, ApiError> {
    let ballot: Ballot = io::from_bytes(&body)?;

    let mut election = state.lock().unwrap();
    if election.server_key.is_none() {
        return Err(not_open());
    }
    election.ballots.push(ballot);
    tracing::info!(total = election.ballots.len(), "ballot received");
    Ok(StatusCode::CREATED)
}

/// `POST /tally` — aggregates every ballot received so far.
async fn run_tally(State(state): State<AppState>) -> Result<String, ApiError> {
    // Copy what we need and release the lock (end of this block) before the
    // long computation, so voters are not blocked while we tally.
    let (server_key, ballots) = {
        let election = state.lock().unwrap();
        let server_key = election.server_key.clone().ok_or_else(not_open)?;
        (server_key, election.ballots.clone())
    };

    let num_ballots = ballots.len();
    let counts = tokio::task::spawn_blocking(move || tally::tally(&server_key, &ballots)).await?;

    state.lock().unwrap().counts = Some(counts);
    tracing::info!(num_ballots, "tally complete");
    Ok(format!("tallied {num_ballots} ballots"))
}

/// `GET /results` — bincode `Vec<FheUint32>` from the latest tally.
/// Only the holder of the client key can decrypt it.
async fn results(State(state): State<AppState>) -> Result<Vec<u8>, ApiError> {
    let election = state.lock().unwrap();
    let counts = election
        .counts
        .as_ref()
        .ok_or_else(|| ApiError::new(StatusCode::NOT_FOUND, "no tally has been run yet"))?;
    Ok(io::to_bytes(counts)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Method, Request};
    use blindtally_core::election::Candidate;
    use tfhe::ClientKey;
    use tfhe::prelude::*;
    // `oneshot` sends one request straight into the router: no port, no
    // network, but the same routing, extractors and error handling.
    use tower::ServiceExt;

    async fn send(app: &Router, method: Method, uri: &str, body: Vec<u8>) -> (StatusCode, Vec<u8>) {
        let request = Request::builder()
            .method(method)
            .uri(uri)
            .body(Body::from(body))
            .unwrap();
        // Cloning a Router is cheap and shares its state (the Arc), so
        // requests sent to clones all see the same election.
        let response = app.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        (status, bytes.to_vec())
    }

    #[tokio::test]
    async fn unknown_route_is_not_found() {
        let (status, _) = send(&router(), Method::GET, "/nope", vec![]).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn wrong_method_is_rejected() {
        let (status, _) = send(&router(), Method::GET, "/ballots", vec![]).await;
        assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn garbage_ballot_is_a_bad_request() {
        let (status, _) = send(&router(), Method::POST, "/ballots", b"garbage".to_vec()).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn garbage_key_is_a_bad_request() {
        let (status, _) = send(&router(), Method::POST, "/keys", b"garbage".to_vec()).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn tally_before_open_is_a_conflict() {
        let (status, body) = send(&router(), Method::POST, "/tally", vec![]).await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert!(String::from_utf8(body).unwrap().contains("not open"));
    }

    #[tokio::test]
    async fn results_before_tally_are_not_found() {
        let (status, _) = send(&router(), Method::GET, "/results", vec![]).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    /// A whole election over the HTTP API, as the client binary runs it.
    #[tokio::test]
    async fn full_election() {
        let app = router();
        let client_key = ClientKey::generate(tfhe::ConfigBuilder::default().build());
        let key_bytes = io::to_bytes(&CompressedServerKey::new(&client_key)).unwrap();

        // A ballot is well-formed but the election has no key yet.
        let ballot = |choice| io::to_bytes(&Ballot::try_new(choice, &client_key).unwrap()).unwrap();
        let (status, _) = send(&app, Method::POST, "/ballots", ballot(Candidate::Alice)).await;
        assert_eq!(status, StatusCode::CONFLICT);

        let (status, _) = send(&app, Method::POST, "/keys", key_bytes.clone()).await;
        assert_eq!(status, StatusCode::CREATED);
        let (status, _) = send(&app, Method::POST, "/keys", key_bytes).await;
        assert_eq!(status, StatusCode::CONFLICT, "the key cannot be replaced");

        for choice in [Candidate::Bob, Candidate::Alice, Candidate::Bob] {
            let (status, _) = send(&app, Method::POST, "/ballots", ballot(choice)).await;
            assert_eq!(status, StatusCode::CREATED);
        }

        let (status, body) = send(&app, Method::POST, "/tally", vec![]).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, b"tallied 3 ballots");

        let (status, body) = send(&app, Method::GET, "/results", vec![]).await;
        assert_eq!(status, StatusCode::OK);
        let counts: Vec<FheUint32> = io::from_bytes(&body).unwrap();
        let counts: Vec<u32> = counts.iter().map(|c| c.decrypt(&client_key)).collect();
        assert_eq!(counts, vec![1, 2]);
    }
}
