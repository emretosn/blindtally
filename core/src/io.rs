use std::fs;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::errors::AppError;

pub fn serialize_to_file<T: ?Sized + Serialize>(path: &str, value: &T) -> Result<(), AppError> {
    let bytes = bincode::serde::encode_to_vec(value, bincode::config::standard())?;
    fs::write(path, bytes)?;
    Ok(())
}

pub fn deserialize_from_file<T: DeserializeOwned>(path: &str) -> Result<T, AppError> {
    let bytes = fs::read(path)?;
    let (value, _) = bincode::serde::decode_from_slice(&bytes, bincode::config::standard())?;
    Ok(value)
}
