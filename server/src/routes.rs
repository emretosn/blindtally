use std::sync::{Arc, Mutex};

use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::Router;
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
    let counts =
        tokio::task::spawn_blocking(move || tally::tally(&server_key, &ballots)).await?;

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
