use clap::Parser;
use std::process::ExitCode;
use ssec_cli::cli::{Cli, Command};
use ssec_cli::{enc, dec};

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
