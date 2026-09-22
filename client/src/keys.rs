use tfhe::{ClientKey, ServerKey};

use blindtally_core::errors::AppError;
use blindtally_core::{io, paths};

/// The client is the only side that ever holds a `ClientKey`, so its
/// path lives here rather than in `blindtally_core::paths` — the
/// server binary has no business knowing where it is.
pub const CLIENT_KEY_PATH: &str = "client_key.bin";

pub fn keygen() -> (ClientKey, ServerKey) {
    let config = tfhe::ConfigBuilder::default().build();
    tfhe::generate_keys(config)
}

pub fn save_keys(client_key: &ClientKey, server_key: &ServerKey) -> Result<(), AppError> {
    io::serialize_to_file(CLIENT_KEY_PATH, client_key)?;
    io::serialize_to_file(paths::SERVER_KEY_PATH, server_key)?;
    Ok(())
}

pub fn load_client_key() -> Result<ClientKey, AppError> {
    io::deserialize_from_file(CLIENT_KEY_PATH)
}
