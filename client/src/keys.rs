use tfhe::{ClientKey, CompressedServerKey};

use blindtally_core::errors::AppError;
use blindtally_core::io;

/// The client is the only side that ever holds a `ClientKey`, so its
/// path lives here — the server binary has no business knowing where it is.
pub const CLIENT_KEY_PATH: &str = "client_key.bin";

pub fn keygen() -> ClientKey {
    let config = tfhe::ConfigBuilder::default().build();
    ClientKey::generate(config)
}

/// The server key is derived from the client key on demand. The compressed
/// form is ~68 MB versus ~190 MB for the full key, which matters over HTTP;
/// the server decompresses it on arrival.
pub fn compressed_server_key(client_key: &ClientKey) -> CompressedServerKey {
    CompressedServerKey::new(client_key)
}

pub fn save_client_key(client_key: &ClientKey) -> Result<(), AppError> {
    io::serialize_to_file(CLIENT_KEY_PATH, client_key)
}

pub fn load_client_key() -> Result<ClientKey, AppError> {
    io::deserialize_from_file(CLIENT_KEY_PATH)
}
