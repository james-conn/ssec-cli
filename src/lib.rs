pub mod cli;

mod file;
mod password;
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

trait GetBufRead: Send + 'static {
	fn get_bufread(&self) -> impl std::io::BufRead;
}

impl GetBufRead for std::io::Stdin {
	fn get_bufread(&self) -> impl std::io::BufRead {
		self.lock()
	}
}

async fn run_with_io(
	cli: cli::Cli,
	reader: impl GetBufRead,
	writer: impl std::io::Write + Send + 'static
) -> std::process::ExitCode {
	match cli.command {
		cli::Command::Enc(args) => handle_err(enc::enc(args, reader, writer).await),
		cli::Command::Dec(args) => handle_err(dec::dec_file(args, reader, writer).await),
		cli::Command::Fetch(args) => handle_err(dec::dec_fetch(args, reader, writer).await)
	}
}

pub async fn run(cli: cli::Cli) -> std::process::ExitCode {
	run_with_io(
		cli,
		std::io::stdin(),
		std::io::stdout()
	).await
}
