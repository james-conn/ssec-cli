use clap::CommandFactory;
use regex::Regex;
use rand::TryRngCore;
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;
use ssec_core::ChaffStream;
use std::str::FromStr;
use crate::cli::{Cli, ChaffArgs};

const SIZE_FORMAT_HELP_MSG: &str = "<number><unit> (where unit is one of 'b', 'B', 'kb', 'KB', 'mb', 'MB', 'gb', 'GB')";

struct HumanSizeParser {
	re: Regex
}

impl Default for HumanSizeParser {
	fn default() -> Self {
		Self {
			re: Regex::new("^([0-9]+)(b|B|kb|KB|mb|MB|gb|GB)$").unwrap()
		}
	}
}

impl HumanSizeParser {
	fn parse(&self, input: &str) -> Result<u64, String> {
		let Some(caps) = self.re.captures(input) else {
			return Err(format!("failed to parse input {input:?}, expected format {SIZE_FORMAT_HELP_MSG}"))
		};

		let Ok(base) = u64::from_str(&caps[1]) else {
			return Err(format!("cannot parse {:?} without integer overflow", &caps[1]));
		};

		let multiplier = match &caps[2] {
			"b" | "B" => 1,
			"kb" | "KB" => 1024,
			"mb" | "MB" => 1024 * 1024,
			"gb" | "GB" => 1024 * 1024 * 1024,
			_ => unreachable!()
		};

		base.checked_mul(multiplier).ok_or_else(|| {
			format!("overflow occurred multiplying base ({base:?}) and multiplier ({multiplier:?})")
		})
	}
}

pub async fn chaff(args: ChaffArgs) -> Result<(), ()> {
	let parser = HumanSizeParser::default();

	let min_size: u64 = match parser.parse(&args.size) {
		Ok(size) => size,
		Err(err) => Cli::command()
			.error(clap::error::ErrorKind::ValueValidation, err)
			.exit()
	};

	let max_size: Option<u64> = args.random_size_max.map(move |s| {
		match parser.parse(&s) {
			Ok(size) => size,
			Err(err) => Cli::command()
				.error(clap::error::ErrorKind::ValueValidation, err)
				.exit()
		}
	});

	if let Some(max_size) = max_size && max_size <= min_size {
		Cli::command()
			.error(
				clap::error::ErrorKind::ValueValidation,
				format!("max size ({max_size:?}) must be greater than min size ({min_size:?})")
			).exit()
	}

	let size = match max_size {
		Some(max_size) => {
			let r = rand::rngs::OsRng.try_next_u64().unwrap();
			// technically this isn't perfectly uniform, but probably good enough
			min_size + (r % (max_size - min_size))
		},
		None => min_size
	};

	let mut f_out = tokio::fs::OpenOptions::new()
		.create(true)
		.write(true)
		.truncate(true)
		.open(&args.out_file).await.map_err(|e| {
			eprintln!("failed to open specified outout file {:?}: {e}", args.out_file);
		}).map(tokio::io::BufWriter::new)?;

	let mut chaff = ChaffStream::new(rand::rngs::OsRng, size as usize, 2048).unwrap();

	while let Some(bytes) = chaff.next().await {
		let b = bytes.unwrap();
		f_out.write_all(&b).await.unwrap();
	}

	f_out.shutdown().await.unwrap();

	Ok(())
}
