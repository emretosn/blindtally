use tfhe::prelude::*;
use tfhe::{FheBool, FheUint32, ServerKey, set_server_key};

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

#[cfg(test)]
mod tests {
    use super::*;
    use blindtally_core::election::Candidate;
    use tfhe::{ClientKey, FheUint8, generate_keys};

    /// Tests play both roles: they hold the client key to encrypt ballots
    /// and decrypt the result, and pass only the server key to `tally`.
    fn keys() -> (ClientKey, ServerKey) {
        generate_keys(tfhe::ConfigBuilder::default().build())
    }

    fn decrypt(counts: &[FheUint32], client_key: &ClientKey) -> Vec<u32> {
        counts
            .iter()
            .map(|count| count.decrypt(client_key))
            .collect()
    }

    #[test]
    fn counts_each_candidates_votes() {
        let (client_key, server_key) = keys();
        let ballots: Vec<Ballot> = [Candidate::Alice, Candidate::Bob, Candidate::Alice]
            .into_iter()
            .map(|choice| Ballot::try_new(choice, &client_key).unwrap())
            .collect();

        let counts = tally(&server_key, &ballots);

        assert_eq!(decrypt(&counts, &client_key), vec![2, 1]);
    }

    #[test]
    fn no_ballots_means_zero_votes_each() {
        let (client_key, server_key) = keys();

        let counts = tally(&server_key, &[]);

        assert_eq!(decrypt(&counts, &client_key), vec![0; NUM_CANDIDATES]);
    }

    #[test]
    fn unknown_candidate_id_counts_for_nobody() {
        let (client_key, server_key) = keys();
        let ballots = vec![
            Ballot::try_new(Candidate::Bob, &client_key).unwrap(),
            Ballot {
                encrypted_choice: FheUint8::encrypt(7u8, &client_key),
            },
        ];

        let counts = tally(&server_key, &ballots);

        assert_eq!(decrypt(&counts, &client_key), vec![0, 1]);
    }
}
