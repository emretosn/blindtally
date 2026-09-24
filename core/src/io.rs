use std::fs;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::errors::AppError;

/// Encodes a value into bincode bytes, e.g. for an HTTP request body.
pub fn to_bytes<T: ?Sized + Serialize>(value: &T) -> Result<Vec<u8>, AppError> {
    Ok(bincode::serde::encode_to_vec(value, bincode::config::standard())?)
}

/// Decodes a value from bincode bytes produced by [`to_bytes`].
pub fn from_bytes<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, AppError> {
    let (value, _) = bincode::serde::decode_from_slice(bytes, bincode::config::standard())?;
    Ok(value)
}

pub fn serialize_to_file<T: ?Sized + Serialize>(path: &str, value: &T) -> Result<(), AppError> {
    fs::write(path, to_bytes(value)?)?;
    Ok(())
}

pub fn deserialize_from_file<T: DeserializeOwned>(path: &str) -> Result<T, AppError> {
    from_bytes(&fs::read(path)?)
}
