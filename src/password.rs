use zeroize::Zeroizing;
use crate::io::IoBundle;

const PASSWORD_PROMPT: &str = "password: ";

pub async fn prompt_password<B: IoBundle>(io: B) -> Result<Zeroizing<Vec<u8>>, std::io::Error> {
	tokio::task::spawn_blocking(move || {
		match B::is_interactive() {
			true => rpassword::prompt_password(PASSWORD_PROMPT),
			false => {
				rpassword::prompt_password_from_bufread(&mut io.get_bufread(), &mut io.get_write(), PASSWORD_PROMPT)
			}
		}.map(String::into_bytes).map(Zeroizing::new)
	}).await.unwrap()
}
