mod tally;

use clap::{Parser, Subcommand};
use tfhe::ServerKey;

use blindtally_core::election::Ballot;
use blindtally_core::errors::AppError;
use blindtally_core::{io, paths};

#[derive(Parser)]
#[command(name = "blindtally-server", about = "Aggregate encrypted ballots")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Aggregate all ballots into encrypted counts
    Tally,
}

fn main() -> Result<(), AppError> {
    let cli = Cli::parse();
    match cli.command {
        Command::Tally => cmd_tally(),
    }
}

fn cmd_tally() -> Result<(), AppError> {
    let ballots: Vec<Ballot> = io::deserialize_from_file(paths::BALLOTS_PATH)?;
    let server_key: ServerKey = io::deserialize_from_file(paths::SERVER_KEY_PATH)?;

    let counts = tally::tally(&server_key, &ballots);
    io::serialize_to_file(paths::COUNTS_PATH, &counts)?;

    println!("tallied {} ballots", ballots.len());
    Ok(())
}
