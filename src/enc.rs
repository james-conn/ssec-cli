use ssec_core::Encrypt;
use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;
use zeroize::Zeroizing;
use rand::rngs::OsRng;
use indicatif::{ProgressBar, ProgressStyle};
use crate::cli::EncArgs;
use crate::BAR_STYLE;

const SPINNER_STYLE: &str = "{spinner} deriving encryption key";

pub async fn enc(args: EncArgs) -> Result<(), ()> {
	let password = tokio::task::spawn_blocking(move || {
		rpassword::prompt_password("password: ")
			.map(Zeroizing::new)
	}).await.unwrap().map_err(|e| {
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

	let progress = ProgressBar::new(f_in_len);
	progress.set_style(ProgressStyle::with_template(BAR_STYLE).unwrap());
	let f_in = progress.wrap_async_read(f_in);

	let s = tokio_util::io::ReaderStream::new(f_in);
	let mut enc = tokio::task::spawn_blocking(move || {
		let spinner = ProgressBar::new_spinner();
		spinner.set_style(ProgressStyle::with_template(SPINNER_STYLE).unwrap());
		spinner.enable_steady_tick(std::time::Duration::from_millis(100));
		Encrypt::new_uncompressed(s, password.as_bytes(), &mut OsRng, f_in_len)
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

	progress.reset();

	while let Some(bytes) = enc.next().await {
		f_out.write_all(&bytes.unwrap()).await.unwrap();
	}

	f_out.shutdown().await.unwrap();

	Ok(())
}
