use zeroize::Zeroizing;
use crate::GetBufRead;

const PASSWORD_PROMPT: &str = "password: ";

pub async fn prompt_password<R: GetBufRead>(
	reader: R,
	mut writer: impl std::io::Write + Send + 'static
) -> Result<Zeroizing<Vec<u8>>, std::io::Error> {
	tokio::task::spawn_blocking(move || {
		match R::is_stdin() {
			true => rpassword::prompt_password(PASSWORD_PROMPT),
			false => {
				let mut bufread = reader.get_bufread();
				rpassword::prompt_password_from_bufread(&mut bufread, &mut writer, PASSWORD_PROMPT)
			}
		}.map(String::into_bytes).map(Zeroizing::new)
	}).await.unwrap()
}
