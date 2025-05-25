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
use crate::{DEFINITE_BAR_STYLE, INDEFINITE_BAR_STYLE};

const SPINNER_STYLE: &str = "{spinner} deriving decryption key";

async fn dec_stream_to<E, S>(
	stream: S,
	password: Zeroizing<Vec<u8>>,
	out_path: PathBuf,
	show_progress: bool,
	enc_len: Option<u64>
) -> Result<(), ()>
where
	E: std::error::Error,
	S: Stream<Item = Result<bytes::Bytes, E>> + Unpin + Send + 'static
{
	let progress = match show_progress {
		true => ProgressBar::new_spinner(),
		false => ProgressBar::hidden()
	};
	let stream = stream.map({
		let progress = progress.clone();
		move |b| {
			if let Ok(b) = &b {
				progress.inc(b.len() as u64);
			}
			b
		}
	});

	let (dec, f_out) = tokio::join!(
		async {
			let dec = Decrypt::new(stream).await.unwrap();
			tokio::task::spawn_blocking({
				let progress = progress.clone();
				move || {
					progress.set_style(ProgressStyle::with_template(SPINNER_STYLE).unwrap());
					progress.enable_steady_tick(std::time::Duration::from_millis(100));

					dec.try_password(&password)
				}
			}).await.unwrap()
		},
		new_async_tempfile()
	);

	let mut dec = match dec {
		Ok(dec) => dec,
		Err(_) => {
			progress.suspend(|| {
				eprintln!("password incorrect");
			});
			return Err(());
		}
	};
	let mut f_out = f_out.unwrap();

	progress.disable_steady_tick();
	match enc_len {
		Some(enc_len) => {
			progress.set_length(enc_len);
			progress.set_style(ProgressStyle::with_template(DEFINITE_BAR_STYLE).unwrap());
		},
		None => progress.set_style(ProgressStyle::with_template(INDEFINITE_BAR_STYLE).unwrap())
	}
	progress.reset();

	while let Some(bytes) = dec.next().await {
		let b = match bytes {
			Ok(b) => b,
			Err(e) => {
				progress.suspend(|| {
					eprintln!("{e}");
				});
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

	let f_in_metadata = f_in.metadata().await.map_err(|e| {
		eprintln!("failed to get metadata of input file: {e}");
	})?;

	let s = tokio_util::io::ReaderStream::new(f_in);

	dec_stream_to(
		s,
		password,
		args.out_file,
		B::is_interactive() && !args.silent,
		Some(f_in_metadata.len())
	).await
}

pub async fn dec_fetch<B: IoBundle>(args: FetchArgs, io: B) -> Result<(), ()> {
	let password = prompt_password(io).await.map_err(|e| {
		eprintln!("failed to read password interactively: {e}");
	})?;

	let client = reqwest::Client::new();

	let resp = client.get(args.url.clone()).send().await.map_err(|e| {
		eprintln!("failed to fetch remote file {:?}: {e}", args.url);
	})?;
	let enc_len = resp.content_length();
	let s = resp.bytes_stream();

	dec_stream_to(
		s,
		password,
		args.out_file,
		B::is_interactive() && !args.silent,
		enc_len
	).await
}
