use thiserror::Error;

/// Application-wide error enum. `#[from]` auto-implements `From`,
/// so `?` can convert any of these library errors into `AppError`.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("tfhe: {0}")]
    Tfhe(#[from] tfhe::Error),
    #[error("serialization (encode): {0}")]
    Encode(#[from] bincode::error::EncodeError),
    #[error("serialization (decode): {0}")]
    Decode(#[from] bincode::error::DecodeError),
}
