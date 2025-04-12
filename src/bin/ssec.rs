use clap::Parser;
use ssec_cli::{cli::Cli, run};

#[tokio::main]
async fn main() -> std::process::ExitCode {
	let cli = Cli::parse();
	run(cli).await
}
