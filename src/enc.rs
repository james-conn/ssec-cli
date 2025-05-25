use ssec_core::Encrypt;
use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;
use rand::rngs::OsRng;
use indicatif::{ProgressBar, ProgressStyle};
use crate::cli::EncArgs;
use crate::password::prompt_password;
use crate::io::IoBundle;
use crate::DEFINITE_BAR_STYLE;

const SPINNER_STYLE: &str = "{spinner} deriving encryption key";

pub async fn enc<B: IoBundle>(args: EncArgs, io: B) -> Result<(), ()> {
	let password = prompt_password(io).await.map_err(|e| {
		eprintln!("failed to read password interactively: {e}");
	})?;

	let f_in = tokio::fs::File::open(&args.in_file).await.map_err(|e| {
		eprintln!("failed to open file {:?}: {e}", args.in_file);
	})?;
	let f_in_metadata = f_in.metadata().await.map_err(|e| {
		eprintln!("failed to get metadata of input file: {e}");
	})?;

	if !f_in_metadata.is_file() {
		eprintln!("ERROR: input file {:?} is not, in fact, a file", args.in_file);
		return Err(());
	}

	let f_in_len = f_in_metadata.len();
	if f_in_len == 0 {
		eprintln!("input file {:?} is empty", args.in_file);
		return Ok(());
	}

	let progress = match B::is_interactive() && !args.silent {
		true => ProgressBar::new(f_in_len),
		false => ProgressBar::hidden()
	};
	let progress_read = progress.wrap_async_read(f_in);
	let s = tokio_util::io::ReaderStream::new(progress_read);
	let mut enc = tokio::task::spawn_blocking(move || {
		let spinner = match B::is_interactive() && !args.silent {
			true => ProgressBar::new_spinner(),
			false => ProgressBar::hidden()
		};
		spinner.set_style(ProgressStyle::with_template(SPINNER_STYLE).unwrap());
		spinner.enable_steady_tick(std::time::Duration::from_millis(100));

		Encrypt::new_uncompressed(s, &password, &mut OsRng)
	}).await.unwrap().unwrap();

	let mut f_out = match args.out_file {
		Some(out_file) => {
			tokio::fs::OpenOptions::new()
				.create(true)
				.write(true)
				.truncate(true)
				.open(&out_file).await.map_err(|e| {
					eprintln!("failed to open specified outout file {out_file:?}: {e}");
				}).map(tokio::io::BufWriter::new)?
		},
		None => {
			let out_name = format!("{}.ssec", args.in_file.display());
			tokio::fs::File::create_new(&out_name).await.map_err(|e| {
				eprintln!("failed to create new output file {out_name:?}: {e}");
			}).map(tokio::io::BufWriter::new)?
		}
	};

	progress.set_style(ProgressStyle::with_template(DEFINITE_BAR_STYLE).unwrap());
	progress.reset();

	while let Some(bytes) = enc.next().await {
		let b = bytes.unwrap();
		f_out.write_all(&b).await.unwrap();
	}

	f_out.shutdown().await.unwrap();

	Ok(())
}
