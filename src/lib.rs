#![forbid(unsafe_code)]

pub mod cli;

mod file;
mod io;
mod password;
mod enc;
mod dec;
mod chaff;

#[cfg(test)]
mod tests;

const DEFINITE_BAR_STYLE: &str = "[{elapsed_precise}] {binary_bytes_per_sec} {bar} {binary_bytes}/{binary_total_bytes} ({eta})";
const INDEFINITE_BAR_STYLE: &str = "[{elapsed_precise}] {binary_bytes_per_sec} ({eta})";

#[inline]
fn handle_err(result: Result<(), ()>) -> std::process::ExitCode {
	match result {
		Ok(()) => std::process::ExitCode::SUCCESS,
		Err(()) => std::process::ExitCode::FAILURE
	}
}

async fn run_with_io<B: io::IoBundle>(cli: cli::Cli, io: B) -> std::process::ExitCode {
	match cli.command {
		cli::Command::Enc(args) => handle_err(enc::enc(args, io).await),
		cli::Command::Dec(args) => handle_err(dec::dec_file(args, io).await),
		cli::Command::Fetch(args) => handle_err(dec::dec_fetch(args, io).await),
		cli::Command::Chaff(args) => handle_err(chaff::chaff(args).await),
	}
}

pub async fn run(cli: cli::Cli) -> std::process::ExitCode {
	run_with_io(cli, io::InteractiveIo).await
}
