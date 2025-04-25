use ssec_core::Decrypt;
use futures_util::{Stream, StreamExt};
use tokio::io::AsyncWriteExt;
use zeroize::Zeroizing;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use crate::cli::{DecArgs, FetchArgs};
use crate::file::new_async_tempfile;
use crate::password::prompt_password;
use crate::io::IoBundle;
use crate::BAR_STYLE;

const SPINNER_STYLE: &str = "{spinner} deriving decryption key";

async fn dec_stream_to<E, S>(
	stream: S,
	password: Zeroizing<Vec<u8>>,
	out_path: PathBuf,
	is_interactive: bool
) -> Result<(), ()>
where
	E: std::error::Error,
	S: Stream<Item = Result<bytes::Bytes, E>> + Unpin + Send + 'static
{
	let (dec, f_out) = tokio::join!(
		async {
			let dec = Decrypt::new(stream).await.unwrap();
			tokio::task::spawn_blocking(move || {
				let spinner = match is_interactive {
					true => ProgressBar::new_spinner(),
					false => ProgressBar::hidden()
				};
				spinner.set_style(ProgressStyle::with_template(SPINNER_STYLE).unwrap());
				spinner.enable_steady_tick(std::time::Duration::from_millis(100));

				dec.try_password(&password)
			}).await.unwrap()
		},
		new_async_tempfile()
	);

	let mut dec = match dec {
		Ok(dec) => dec,
		Err(_) => {
			eprintln!("password incorrect");
			return Err(());
		}
	};
	let mut f_out = f_out.unwrap();

	let total = dec.remaining_read_len();
	let progress = match is_interactive {
		true => ProgressBar::new(total),
		false => ProgressBar::hidden()
	};
	progress.set_style(ProgressStyle::with_template(BAR_STYLE).unwrap());

	while let Some(bytes) = dec.next().await {
		progress.set_position(total - dec.remaining_read_len());

		let b = match bytes {
			Ok(b) => b,
			Err(e) => {
				eprintln!("{e}");
				return Err(());
			},
		};

		f_out.as_mut().write_all(&b).await.unwrap();
	}

	f_out.as_mut().shutdown().await.unwrap();

	f_out.persist(out_path).await.unwrap();

	Ok(())
}

pub async fn dec_file<B: IoBundle>(args: DecArgs, io: B) -> Result<(), ()> {
	let password = prompt_password(io).await.map_err(|e| {
		eprintln!("failed to read password interactively: {e}");
	})?;

	let f_in = tokio::fs::File::open(&args.in_file).await.map_err(|e| {
		eprintln!("failed to open file {:?}: {e}", args.in_file);
	})?;
	let s = tokio_util::io::ReaderStream::new(f_in);

	dec_stream_to(s, password, args.out_file, B::is_interactive()).await
}

pub async fn dec_fetch<B: IoBundle>(args: FetchArgs, io: B) -> Result<(), ()> {
	let password = prompt_password(io).await.map_err(|e| {
		eprintln!("failed to read password interactively: {e}");
	})?;

	let client = reqwest::Client::new();

	let s = client.get(args.url.clone()).send().await.map_err(|e| {
		eprintln!("failed to fetch remote file {:?}: {e}", args.url);
	})?.bytes_stream();

	dec_stream_to(s, password, args.out_file, B::is_interactive()).await
}
