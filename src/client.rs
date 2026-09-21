use tfhe::{ClientKey, ServerKey};

use crate::election::{Ballot, Candidate};
use crate::errors::AppError;
use crate::utils;

/// All operations that require the *secret* key live here.
/// This module is the "client" in the FHE client/server model.
pub const CLIENT_KEY_PATH: &str = "client_key.bin";
pub const TALLY_SEED_PATH: &str = "tally_seed.bin";

pub fn keygen() -> (ClientKey, ServerKey) {
    let config = tfhe::ConfigBuilder::default().build();
    tfhe::generate_keys(config)
}

pub fn save_keys(client_key: &ClientKey, server_key: &ServerKey) -> Result<(), AppError> {
    utils::serialize_to_file(CLIENT_KEY_PATH, client_key)?;
    utils::serialize_to_file(utils::SERVER_KEY_PATH, server_key)?;
    Ok(())
}

pub fn load_client_key() -> Result<ClientKey, AppError> {
    utils::deserialize_from_file(CLIENT_KEY_PATH)
}

pub fn encrypt_votes(votes: &[Candidate], client_key: &ClientKey) -> Result<Vec<Ballot>, AppError> {
    Ok(votes
        .iter()
        .map(|c| Ballot::try_new(*c, client_key))
        .collect::<Result<Vec<_>, _>>()?)
}
