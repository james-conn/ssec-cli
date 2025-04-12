use clap_complete::{generate_to, Shell};
use clap::{ValueEnum, CommandFactory};
use std::path::PathBuf;

include!("src/cli.rs");

const ENV_VAR: &str = "SHELLS_OUTPUT";

fn main() {
	println!("cargo::rerun-if-env-changed={ENV_VAR}");

	if let Some(mut shells_out) = std::env::var_os(ENV_VAR).map(PathBuf::from) {
		if let Some(mut out) = std::env::var_os("out").map(PathBuf::from) {
			out.push(shells_out);
			shells_out = out;
		}

		let mut cmd = Cli::command();

		for &shell in Shell::value_variants() {
			generate_to(shell, &mut cmd, "ssec", &shells_out).unwrap();
		}
	}
}
