use tfhe::prelude::*;
use tfhe::{set_server_key, FheBool, FheUint32, ServerKey};

use crate::election::{Ballot, NUM_CANDIDATES};
use crate::errors::AppError;
use crate::utils;

/// All operations on encrypted data that do NOT need the secret key live here.
pub fn load_server_key() -> Result<ServerKey, AppError> {
    let server_key: ServerKey = utils::deserialize_from_file(utils::SERVER_KEY_PATH)?;
    set_server_key(server_key.clone());
    Ok(server_key)
}

/// Computes encrypted per-candidate counts. Notice: no client key in sight,
/// so this function is physically unable to look at any individual vote.
pub fn tally(ballots: &[Ballot], initial_count: &FheUint32) -> Vec<FheUint32> {
    let mut counts: Vec<FheUint32> = (0..NUM_CANDIDATES).map(|_| initial_count.clone()).collect();

    for ballot in ballots {
        for candidate_id in 0..NUM_CANDIDATES as u8 {
            let is_match: FheBool = ballot.encrypted_choice.eq(candidate_id);
            let one_if_match: FheUint32 = FheUint32::if_then_else(&is_match, 1u32, 0u32);
            counts[candidate_id as usize] += &one_if_match;
        }
    }
    counts
}
