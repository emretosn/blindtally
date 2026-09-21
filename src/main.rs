mod client;
mod election;
mod errors;
mod server;
mod utils;

use tfhe::prelude::*;
use tfhe::FheUint32;

use election::Candidate;

fn main() -> Result<(), errors::AppError> {
    // ------- Client machine: generate keys and publish the public one -------
    let (client_key, server_key) = client::keygen();
    client::save_keys(&client_key, &server_key)?;

    // Client encrypts votes and writes the encrypted ballots to disk
    let votes = [
        Candidate::Alice,
        Candidate::Bob,
        Candidate::Alice,
        Candidate::Alice,
    ];
    let ballots = client::encrypt_votes(&votes, &client_key)?;
    utils::serialize_to_file(utils::BALLOTS_PATH, &ballots)?;

    // The client also provides one encrypted 0 so the server can start counting
    let encrypted_zero = FheUint32::try_encrypt(0u32, &client_key)?;
    utils::serialize_to_file(client::TALLY_SEED_PATH, &encrypted_zero)?;

    // ------- Server machine: reload everything from disk, tally ------
    // Note: the client key is NOT loaded or used in this section.
    let loaded_ballots: Vec<election::Ballot> =
        utils::deserialize_from_file(utils::BALLOTS_PATH)?;
    let seed: FheUint32 = utils::deserialize_from_file(client::TALLY_SEED_PATH)?;

    server::load_server_key()?;
    let encrypted_counts = server::tally(&loaded_ballots, &seed);
    utils::serialize_to_file(utils::COUNTS_PATH, &encrypted_counts)?;

    // ------- Client machine: publish results ------
    let client_key_loaded = client::load_client_key()?;
    let counts: Vec<FheUint32> = utils::deserialize_from_file(utils::COUNTS_PATH)?;

    for candidate in [Candidate::Alice, Candidate::Bob] {
        let count: u32 = counts[candidate as usize].decrypt(&client_key_loaded);
        println!("{:?}: {} votes", candidate, count);
    }

    Ok(())
}
