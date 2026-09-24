mod api;
mod error;
mod keys;

use std::process::ExitCode;

use clap::{Parser, Subcommand};
use tfhe::prelude::*;

use blindtally_core::election::{self, Candidate};
use blindtally_core::errors::AppError;

use api::Api;
use error::ClientError;

#[derive(Parser)]
#[command(name = "blindtally-client", about = "Cast and decrypt blind votes")]
struct Cli {
    /// Base URL of the blindtally server
    #[arg(long, global = true, default_value = "http://127.0.0.1:3000")]
    server: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generate the client key (stays local, never sent anywhere)
    Keygen,
    /// Upload the server key to open the election
    Open,
    /// Cast an encrypted vote for a candidate
    Vote {
        /// Name of the candidate, e.g. "alice"
        candidate: String,
    },
    /// Ask the server to aggregate all ballots
    Tally,
    /// Fetch and decrypt the election results
    Result,
}

/// Returning `ExitCode` instead of `Result` lets us print errors with their
/// friendly `Display` text rather than the `Debug` form `main` would use.
fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), ClientError> {
    let api = Api::new(&cli.server)?;
    match cli.command {
        Command::Keygen => cmd_keygen(),
        Command::Open => cmd_open(&api),
        Command::Vote { candidate } => cmd_vote(&api, &candidate),
        Command::Tally => cmd_tally(&api),
        Command::Result => cmd_result(&api),
    }
}

fn cmd_keygen() -> Result<(), ClientError> {
    let client_key = keys::keygen();
    keys::save_client_key(&client_key)?;
    println!("client key written: {}", keys::CLIENT_KEY_PATH);
    Ok(())
}

fn cmd_open(api: &Api) -> Result<(), ClientError> {
    let client_key = keys::load_client_key()?;
    api.upload_key(&keys::compressed_server_key(&client_key))?;
    println!("election opened");
    Ok(())
}

fn cmd_vote(api: &Api, candidate: &str) -> Result<(), ClientError> {
    let candidate: Candidate = candidate
        .parse()
        .map_err(|_| AppError::UnknownCandidate(candidate.to_string()))?;

    let client_key = keys::load_client_key()?;
    let ballot = election::Ballot::try_new(candidate, &client_key)?;
    api.cast_ballot(&ballot)?;

    println!("vote cast for {candidate:?}");
    Ok(())
}

fn cmd_tally(api: &Api) -> Result<(), ClientError> {
    println!("{}", api.tally()?);
    Ok(())
}

fn cmd_result(api: &Api) -> Result<(), ClientError> {
    let client_key = keys::load_client_key()?;
    let counts = api.results()?;

    for candidate in election::ALL_CANDIDATES {
        let count: u32 = counts[candidate as usize].decrypt(&client_key);
        println!("{candidate:?}: {count} votes");
    }
    Ok(())
}
