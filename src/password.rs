use zeroize::Zeroizing;
use crate::io::IoMode;

pub async fn prompt_password(io: IoMode) -> Result<Zeroizing<Vec<u8>>, std::io::Error> {
	tokio::task::spawn_blocking(move || {
		let builder = rpassword::ConfigBuilder::new();

		let config: rpassword::Config = match io {
			IoMode::Interactive => builder.build(),
			#[cfg(test)]
			IoMode::TestMockedInput(mocked_password) => builder
				.input_data(mocked_password)
				.output_discard()
				.build()
		};

		rpassword::prompt_password_with_config("password: ", config)
			.map(String::into_bytes).map(Zeroizing::new)
	}).await.unwrap()
}
