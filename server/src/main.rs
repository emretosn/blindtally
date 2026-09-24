mod error;
mod routes;
mod tally;

use std::net::SocketAddr;

use clap::Parser;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;

#[derive(Parser)]
#[command(name = "blindtally-server", about = "Aggregate encrypted ballots over HTTP")]
struct Cli {
    /// Address to listen on
    #[arg(long, default_value = "127.0.0.1:3000")]
    addr: SocketAddr,
}

/// `#[tokio::main]` starts a tokio runtime and runs this async `main` on it.
#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt().init();
    let cli = Cli::parse();

    // Middleware ("layers") wrap every route; this one logs each request.
    let app = routes::router().layer(
        TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
            .on_response(DefaultOnResponse::new().level(Level::INFO)),
    );

    let listener = tokio::net::TcpListener::bind(cli.addr).await?;
    tracing::info!("listening on http://{}", cli.addr);
    axum::serve(listener, app).await
}
