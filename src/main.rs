use clap::Parser;
use std::process::ExitCode;

mod cli;
use cli::{Cli, Command};

mod file;

mod enc;
mod dec;

pub const BAR_STYLE: &str = "[{elapsed_precise}] {binary_bytes_per_sec} {bar} {binary_bytes}/{binary_total_bytes} ({eta})";

#[inline]
fn handle_err(result: Result<(), ()>) -> ExitCode {
	match result {
		Ok(()) => ExitCode::SUCCESS,
		Err(()) => ExitCode::FAILURE
	}
}

#[tokio::main]
async fn main() -> ExitCode {
	let cli = Cli::parse();

	match cli.command {
		Command::Enc(args) => handle_err(enc::enc(args).await),
		Command::Dec(args) => handle_err(dec::dec_file(args).await),
		Command::Fetch(args) => handle_err(dec::dec_fetch(args).await)
	}
}
