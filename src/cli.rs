use clap::{Parser, Subcommand, Args};

#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
	#[command(subcommand)]
	pub command: Command
}

#[derive(Subcommand)]
pub enum Command {
	Enc(EncArgs),
	Dec(DecArgs),
	Fetch(FetchArgs),
	Chaff(ChaffArgs)
}

#[derive(Args)]
pub struct EncArgs {
	#[arg(value_hint = clap::ValueHint::FilePath)]
	pub in_file: std::path::PathBuf,

	#[arg(value_hint = clap::ValueHint::FilePath)]
	pub out_file: Option<std::path::PathBuf>,

	#[arg(long)]
	pub silent: bool
}

#[derive(Args)]
pub struct DecArgs {
	#[arg(value_hint = clap::ValueHint::FilePath)]
	pub in_file: std::path::PathBuf,

	#[arg(value_hint = clap::ValueHint::FilePath)]
	pub out_file: std::path::PathBuf,

	#[arg(long)]
	pub silent: bool
}

#[derive(Args)]
pub struct FetchArgs {
	pub url: url::Url,

	#[arg(value_hint = clap::ValueHint::FilePath)]
	pub out_file: std::path::PathBuf,

	#[arg(long)]
	pub silent: bool
}

#[derive(Args)]
pub struct ChaffArgs {
	#[arg(value_hint = clap::ValueHint::FilePath)]
	pub out_file: std::path::PathBuf,

	#[arg(long)]
	pub size: String,

	#[arg(long, requires = "size")]
	pub random_size_max: Option<String>,

	#[arg(long)]
	pub silent: bool
}
