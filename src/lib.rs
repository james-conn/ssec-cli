pub mod cli;

mod file;
mod enc;
mod dec;

const BAR_STYLE: &str = "[{elapsed_precise}] {binary_bytes_per_sec} {bar} {binary_bytes}/{binary_total_bytes} ({eta})";

#[inline]
fn handle_err(result: Result<(), ()>) -> std::process::ExitCode {
	match result {
		Ok(()) => std::process::ExitCode::SUCCESS,
		Err(()) => std::process::ExitCode::FAILURE
	}
}

pub async fn run(cli: cli::Cli) -> std::process::ExitCode {
	match cli.command {
		cli::Command::Enc(args) => handle_err(enc::enc(args).await),
		cli::Command::Dec(args) => handle_err(dec::dec_file(args).await),
		cli::Command::Fetch(args) => handle_err(dec::dec_fetch(args).await)
	}
}
