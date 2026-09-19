use tfhe::prelude::*;
use tfhe::{generate_keys, set_server_key, ClientKey, ConfigBuilder, FheBool, FheUint8, FheUint32};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Candidate {
    Alice = 0,
    Bob = 1,
    Carol = 2,
}

const NUM_CANDIDATES: usize = 3;

#[derive(Clone)]
struct Ballot {
    encrypted_choice: FheUint8,
}

impl Ballot {
    fn try_new(choice: Candidate, client_key: &ClientKey) -> tfhe::Result<Self> {
        Ok(Ballot {
            encrypted_choice: FheUint8::try_encrypt(choice as u8, client_key)?,
        })
    }
}

/// Computes encrypted per-candidate counts. Notice: no client key in sight,
/// so this function is *physically unable* to look at any individual vote.
fn tally(ballots: &[Ballot], initial_count: &FheUint32) -> Vec<FheUint32> {
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ConfigBuilder::default().build();
    let (client_key, server_keys) = generate_keys(config);

    // Client side: voters encrypt their choices
    let votes = [
        (Candidate::Alice),
        (Candidate::Bob),
        (Candidate::Alice),
        (Candidate::Carol),
        (Candidate::Alice),
    ];
    let ballots: Vec<Ballot> = votes
        .iter()
        .map(|c| Ballot::try_new(*c, &client_key))
        .collect::<Result<Vec<_>, _>>()?;

    let encrypted_zero = FheUint32::try_encrypt(0u32, &client_key)?;

    set_server_key(server_keys);

    let encrypted_counts = tally(&ballots, &encrypted_zero);

    for candidate in [Candidate::Alice, Candidate::Bob, Candidate::Carol] {
        let count: u32 = encrypted_counts[candidate as usize].decrypt(&client_key);
        println!("{:?}: {} votes", candidate, count);
    }
Ok(()) }


