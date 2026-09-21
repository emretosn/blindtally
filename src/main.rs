mod client;
mod election;
mod errors;
mod server;
mod utils;

use clap::{Parser, Subcommand};
use tfhe::prelude::*;
use tfhe::FheUint32;

use election::Candidate;
use errors::AppError;

#[derive(Parser)]
#[command(name = "blindtally", about = "Blind voting over fully homomorphic encryption")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generate the client and server keys
    Keygen,
    /// Cast an encrypted vote for a candidate
    Vote {
        /// Name of the candidate, e.g. "alice"
        candidate: String,
    },
    /// Aggregate all ballots into encrypted counts
    Tally,
    /// Decrypt and print the election results
    Result,
}

fn main() -> Result<(), AppError> {
    let cli = Cli::parse();
    match cli.command {
        Command::Keygen => cmd_keygen(),
        Command::Vote { candidate } => cmd_vote(&candidate),
        Command::Tally => cmd_tally(),
        Command::Result => cmd_result(),
    }
}

fn cmd_keygen() -> Result<(), AppError> {
    let (client_key, server_key) = client::keygen();
    client::save_keys(&client_key, &server_key)?;

    let encrypted_zero = FheUint32::try_encrypt(0u32, &client_key)?;
    utils::serialize_to_file(client::TALLY_SEED_PATH, &encrypted_zero)?;

    println!(
        "keys written: {}, {}",
        client::CLIENT_KEY_PATH,
        utils::SERVER_KEY_PATH
    );
    Ok(())
}

fn cmd_vote(candidate: &str) -> Result<(), AppError> {
    let candidate: Candidate = candidate
        .parse()
        .map_err(|_| AppError::UnknownCandidate(candidate.to_string()))?;

    let client_key = client::load_client_key()?;
    let ballot = election::Ballot::try_new(candidate, &client_key)?;

    let mut ballots: Vec<election::Ballot> =
        utils::deserialize_from_file(utils::BALLOTS_PATH).unwrap_or_default();
    ballots.push(ballot);
    utils::serialize_to_file(utils::BALLOTS_PATH, &ballots)?;

    println!("vote cast for {candidate:?}");
    Ok(())
}

fn cmd_tally() -> Result<(), AppError> {
    let ballots: Vec<election::Ballot> = utils::deserialize_from_file(utils::BALLOTS_PATH)?;
    let seed: FheUint32 = utils::deserialize_from_file(client::TALLY_SEED_PATH)?;

    server::load_server_key()?;
    let counts = server::tally(&ballots, &seed);
    utils::serialize_to_file(utils::COUNTS_PATH, &counts)?;

    println!("tallied {} ballots", ballots.len());
    Ok(())
}

fn cmd_result() -> Result<(), AppError> {
    let client_key = client::load_client_key()?;
    let counts: Vec<FheUint32> = utils::deserialize_from_file(utils::COUNTS_PATH)?;

    for candidate in election::ALL_CANDIDATES {
        let count: u32 = counts[candidate as usize].decrypt(&client_key);
        println!("{candidate:?}: {count} votes");
    }
    Ok(())
}
