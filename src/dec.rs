use ssec_core::Decrypt;
use futures_util::{Stream, StreamExt};
use tokio::io::AsyncWriteExt;
use zeroize::Zeroizing;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use crate::cli::{DecArgs, FetchArgs};
use crate::file::new_async_tempfile;
use crate::BAR_STYLE;

const SPINNER_STYLE: &str = "{spinner} deriving decryption key";

async fn dec_stream_to<E: std::error::Error, S: Stream<Item = Result<bytes::Bytes, E>> + Unpin>(
	stream: S,
	password: Zeroizing<Vec<u8>>,
	out_path: PathBuf
) -> Result<(), ()> {
	let mut dec = Decrypt::new(stream, password);

	let mut f_out = new_async_tempfile().await.unwrap();

	let mut total = None;
	let progress = ProgressBar::new_spinner();
	progress.set_style(ProgressStyle::with_template(SPINNER_STYLE).unwrap());
	progress.enable_steady_tick(std::time::Duration::from_millis(100));

	while let Some(bytes) = dec.next().await {
		if let Some(remaining) = dec.remaining_read_len() {
			match total {
				Some(total) => progress.set_position(total - remaining),
				None => {
					progress.disable_steady_tick();
					progress.set_style(ProgressStyle::with_template(BAR_STYLE).unwrap());
					progress.set_length(remaining);
					progress.reset();
					total = Some(remaining);
				}
			}
		}

		f_out.as_mut().write_all(&bytes.unwrap()).await.unwrap();
	}

	f_out.as_mut().shutdown().await.unwrap();

	f_out.persist(out_path).await.unwrap();

	Ok(())
}

async fn prompt_password() -> Result<Zeroizing<Vec<u8>>, ()> {
	tokio::task::spawn_blocking(move || {
		rpassword::prompt_password("password: ")
			.map(String::into_bytes)
			.map(Zeroizing::new)
	}).await.unwrap().map_err(|e| {
		eprintln!("failed to read password interactively: {e}");
	})
}

pub async fn dec_file(args: DecArgs) -> Result<(), ()> {
	let password = prompt_password().await?;

	let f_in = tokio::fs::File::open(&args.in_file).await.map_err(|e| {
		eprintln!("failed to open file {:?}: {e}", args.in_file);
	})?;
	let s = tokio_util::io::ReaderStream::new(f_in);

	dec_stream_to(s, password, args.out_file).await
}

pub async fn dec_fetch(args: FetchArgs) -> Result<(), ()> {
	let password = prompt_password().await?;

	let client = reqwest::Client::new();

	let s = client.get(args.url.clone()).send().await.map_err(|e| {
		eprintln!("failed to fetch remote file {:?}: {e}", args.url);
	})?.bytes_stream();

	dec_stream_to(s, password, args.out_file).await
}
