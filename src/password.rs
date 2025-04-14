use zeroize::Zeroizing;
use crate::GetBufRead;

pub async fn prompt_password(
	reader: impl GetBufRead,
	mut writer: impl std::io::Write + Send + 'static
) -> Result<Zeroizing<Vec<u8>>, std::io::Error> {
	tokio::task::spawn_blocking(move || {
		let mut bufread = reader.get_bufread();
		rpassword::prompt_password_from_bufread(&mut bufread, &mut writer, "password: ")
			.map(String::into_bytes)
			.map(Zeroizing::new)
	}).await.unwrap()
}
