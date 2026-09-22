use tfhe::{ClientKey, ServerKey};

use blindtally_core::errors::AppError;
use blindtally_core::{io, paths};

/// All operations that require the *secret* key live here.
/// This module is the "client" in the FHE client/server model.
pub fn keygen() -> (ClientKey, ServerKey) {
    let config = tfhe::ConfigBuilder::default().build();
    tfhe::generate_keys(config)
}

pub fn save_keys(client_key: &ClientKey, server_key: &ServerKey) -> Result<(), AppError> {
    io::serialize_to_file(paths::CLIENT_KEY_PATH, client_key)?;
    io::serialize_to_file(paths::SERVER_KEY_PATH, server_key)?;
    Ok(())
}

pub fn load_client_key() -> Result<ClientKey, AppError> {
    io::deserialize_from_file(paths::CLIENT_KEY_PATH)
}
