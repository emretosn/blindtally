mod keys;

use clap::{Parser, Subcommand};
use tfhe::prelude::*;
use tfhe::FheUint32;

use blindtally_core::election::{self, Candidate};
use blindtally_core::errors::AppError;
use blindtally_core::{io, paths};

#[derive(Parser)]
#[command(name = "blindtally-client", about = "Cast and decrypt blind votes")]
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
    /// Decrypt and print the election results
    Result,
}

fn main() -> Result<(), AppError> {
    let cli = Cli::parse();
    match cli.command {
        Command::Keygen => cmd_keygen(),
        Command::Vote { candidate } => cmd_vote(&candidate),
        Command::Result => cmd_result(),
    }
}

fn cmd_keygen() -> Result<(), AppError> {
    let (client_key, server_key) = keys::keygen();
    keys::save_keys(&client_key, &server_key)?;

    let encrypted_zero = FheUint32::try_encrypt(0u32, &client_key)?;
    io::serialize_to_file(paths::TALLY_SEED_PATH, &encrypted_zero)?;

    println!(
        "keys written: {}, {}",
        keys::CLIENT_KEY_PATH,
        paths::SERVER_KEY_PATH
    );
    Ok(())
}

fn cmd_vote(candidate: &str) -> Result<(), AppError> {
    let candidate: Candidate = candidate
        .parse()
        .map_err(|_| AppError::UnknownCandidate(candidate.to_string()))?;

    let client_key = keys::load_client_key()?;
    let ballot = election::Ballot::try_new(candidate, &client_key)?;

    let mut ballots: Vec<election::Ballot> =
        io::deserialize_from_file(paths::BALLOTS_PATH).unwrap_or_default();
    ballots.push(ballot);
    io::serialize_to_file(paths::BALLOTS_PATH, &ballots)?;

    println!("vote cast for {candidate:?}");
    Ok(())
}

fn cmd_result() -> Result<(), AppError> {
    let client_key = keys::load_client_key()?;
    let counts: Vec<FheUint32> = io::deserialize_from_file(paths::COUNTS_PATH)?;

    for candidate in election::ALL_CANDIDATES {
        let count: u32 = counts[candidate as usize].decrypt(&client_key);
        println!("{candidate:?}: {count} votes");
    }
    Ok(())
}
