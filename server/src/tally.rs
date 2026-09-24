use tfhe::prelude::*;
use tfhe::{set_server_key, FheBool, FheUint32, ServerKey};

use blindtally_core::election::{Ballot, NUM_CANDIDATES};

/// Computes encrypted per-candidate counts. Notice: no client key in sight,
/// so this function is physically unable to look at any individual vote.
///
/// tfhe keeps the server key in a thread-local, so it is set here on
/// whichever thread ends up running the computation.
pub fn tally(server_key: &ServerKey, ballots: &[Ballot]) -> Vec<FheUint32> {
    set_server_key(server_key.clone());

    // A trivial encryption is a noiseless ciphertext anyone with the server
    // key can make; adding real ciphertexts to it yields real ciphertexts.
    let zero = FheUint32::encrypt_trivial(0u32);
    let mut counts: Vec<FheUint32> = (0..NUM_CANDIDATES).map(|_| zero.clone()).collect();

    for ballot in ballots {
        for candidate_id in 0..NUM_CANDIDATES as u8 {
            let is_match: FheBool = ballot.encrypted_choice.eq(candidate_id);
            let one_if_match: FheUint32 = FheUint32::if_then_else(&is_match, 1u32, 0u32);
            counts[candidate_id as usize] += &one_if_match;
        }
    }
    counts
}
