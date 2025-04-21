use tempfile::{Builder, TempPath, TempDir, PathPersistError};
use tokio::fs::File;
use tokio::io::BufWriter;

// async drop isn't stable yet, but this will suffice for now
pub struct TempTokioFile {
	file: BufWriter<File>,

	// for use in automatically generated `Drop` impl
	temppath: TempPath,
	tempdir: TempDir
}

impl AsMut<BufWriter<File>> for TempTokioFile {
	fn as_mut(&mut self) -> &mut BufWriter<File> {
		&mut self.file
	}
}

impl TempTokioFile {
	pub async fn persist<P: AsRef<std::path::Path> + Send + 'static>(self, new_path: P) -> Result<(), PathPersistError> {
		tokio::task::spawn_blocking(move || {
			drop(self.file);
			let result = self.temppath.persist(new_path);
			drop(self.tempdir);
			result
		}).await.unwrap()
	}
}

#[cfg(unix)]
fn new_tempdir() -> Result<TempDir, std::io::Error> {
	use std::fs::Permissions;
	use std::os::unix::fs::PermissionsExt;

	Builder::new()
		.permissions(Permissions::from_mode(0o700))
		.tempdir()
}

// let's just assume all non-UNIX operating systems will handle this securely
#[cfg(not(unix))]
fn new_tempdir() -> Result<TempDir, std::io::Error> {
	Builder::new().tempdir()
}

pub async fn new_async_tempfile() -> Result<TempTokioFile, std::io::Error> {
	let tempdir = tokio::task::spawn_blocking(new_tempdir).await.unwrap()?;

	let temppath = TempPath::from_path(tempdir.as_ref().join("ssec-temp"));

	let file = tokio::fs::OpenOptions::new()
		.write(true)
		.create(true)
		.truncate(true)
		.open(&*temppath).await
		.map(BufWriter::new)?;

	Ok(TempTokioFile {
		file,
		temppath,
		tempdir
	})
}
