use reqwest::StatusCode;
use thiserror::Error;

use blindtally_core::errors::AppError;

/// Client-side errors. Kept out of blindtally-core so the core (and the
/// server) don't pull in reqwest just to name its error type.
#[derive(Debug, Error)]
pub enum ClientError {
    #[error(transparent)]
    App(#[from] AppError),
    #[error("tfhe: {0}")]
    Tfhe(#[from] tfhe::Error),
    #[error("http: {0}")]
    Http(#[from] reqwest::Error),
    #[error("server replied {status}: {message}")]
    Server { status: StatusCode, message: String },
}
