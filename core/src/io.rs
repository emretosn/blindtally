use std::fs;

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::errors::AppError;

/// Encodes a value into bincode bytes, e.g. for an HTTP request body.
pub fn to_bytes<T: ?Sized + Serialize>(value: &T) -> Result<Vec<u8>, AppError> {
    Ok(bincode::serde::encode_to_vec(
        value,
        bincode::config::standard(),
    )?)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_round_trip() {
        let value = vec![1u32, 2, 3];
        let decoded: Vec<u32> = from_bytes(&to_bytes(&value).unwrap()).unwrap();
        assert_eq!(decoded, value);
    }

    #[test]
    fn garbage_bytes_are_a_decode_error() {
        let result: Result<Vec<u32>, AppError> = from_bytes(&[0xff]);
        assert!(matches!(result, Err(AppError::Decode(_))));
    }

    #[test]
    fn file_round_trip() {
        // Tests run in parallel, so each one needs its own file name.
        let path = std::env::temp_dir().join("blindtally_io_file_round_trip.bin");
        let path = path.to_str().unwrap();

        serialize_to_file(path, &"hello".to_string()).unwrap();
        let decoded: String = deserialize_from_file(path).unwrap();
        fs::remove_file(path).unwrap();

        assert_eq!(decoded, "hello");
    }

    #[test]
    fn missing_file_is_an_io_error() {
        let result: Result<u32, AppError> = deserialize_from_file("does/not/exist.bin");
        assert!(matches!(result, Err(AppError::Io(_))));
    }
}
