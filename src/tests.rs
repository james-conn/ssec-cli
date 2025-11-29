use rand::{SeedableRng, TryRngCore};
use wiremock::{MockServer, Mock, ResponseTemplate, matchers::method};
use crate::cli::{Cli, Command, EncArgs, DecArgs, FetchArgs, ChaffArgs};
use crate::io::IoBundle;
use crate::run_with_io;

const RNG_SEED: u64 = 12345678;

struct MockStdin(&'static str);

impl IoBundle for MockStdin {
	type IoRead = &'static [u8];
	type IoWrite = std::io::Sink;

	fn get_bufread(&self) -> Self::IoRead {
		self.0.as_bytes()
	}

	fn get_write(&self) -> Self::IoWrite {
		std::io::sink()
	}
}

struct EmptyMockStdin;

impl IoBundle for EmptyMockStdin {
	type IoRead = std::io::Empty;
	type IoWrite = std::io::Sink;

	fn get_bufread(&self) -> Self::IoRead {
		std::io::empty()
	}

	fn get_write(&self) -> Self::IoWrite {
		std::io::sink()
	}
}

#[tokio::test]
async fn end_to_end_file() {
	let mut rng = rand::rngs::SmallRng::seed_from_u64(RNG_SEED);
	let tmp = tempfile::tempdir().unwrap();
	let in_path = tmp.as_ref().join("file1");
	let enc_path = tmp.as_ref().join("file.ssec");
	let dec_path = tmp.as_ref().join("file2");

	let mut contents = vec![0; 100000];
	rng.try_fill_bytes(&mut contents).unwrap();
	tokio::fs::write(&in_path, &contents).await.unwrap();

	let result = run_with_io(
		Cli {
			command: Command::Enc(EncArgs {
				in_file: in_path,
				out_file: Some(enc_path.clone()),
				silent: true
			})
		},
		MockStdin("hunter2\n")
	).await;

	assert_eq!(result, std::process::ExitCode::SUCCESS);

	let result = run_with_io(
		Cli {
			command: Command::Dec(DecArgs {
				in_file: enc_path.clone(),
				out_file: dec_path.clone(),
				silent: true
			})
		},
		MockStdin("hunter2\n")
	).await;

	assert_eq!(result, std::process::ExitCode::SUCCESS);

	let dec_contents = tokio::fs::read(&dec_path).await.unwrap();

	assert_eq!(contents, dec_contents);

	let result = run_with_io(
		Cli {
			command: Command::Dec(DecArgs {
				in_file: enc_path,
				out_file: dec_path.clone(),
				silent: true
			})
		},
		MockStdin("not_hunter2\n")
	).await;

	assert_eq!(result, std::process::ExitCode::FAILURE);

	let dec_contents_wrong = tokio::fs::read(&dec_path).await.unwrap();
	assert_eq!(dec_contents, dec_contents_wrong);
}

#[tokio::test]
async fn end_to_end_fetch() {
	let mut rng = rand::rngs::SmallRng::seed_from_u64(RNG_SEED);
	let tmp = tempfile::tempdir().unwrap();
	let in_path = tmp.as_ref().join("in_file");
	let enc_path = tmp.as_ref().join("file.ssec");
	let dec_path = tmp.as_ref().join("out_file");

	let mut contents = vec![0; 20000];
	rng.try_fill_bytes(&mut contents).unwrap();
	tokio::fs::write(&in_path, &contents).await.unwrap();

	let result = run_with_io(
		Cli {
			command: Command::Enc(EncArgs {
				in_file: in_path,
				out_file: Some(enc_path.clone()),
				silent: true
			})
		},
		MockStdin("hunter2\n")
	).await;

	assert_eq!(result, std::process::ExitCode::SUCCESS);

	let mock_server = MockServer::start().await;
	let enc_contents = tokio::fs::read(&enc_path).await.unwrap();

	Mock::given(method("GET"))
		.respond_with(
			ResponseTemplate::new(200)
				.set_body_bytes(enc_contents)
		).mount(&mock_server).await;

	let enc_url = url::Url::parse(&mock_server.uri()).unwrap();

	let result = run_with_io(
		Cli {
			command: Command::Fetch(FetchArgs {
				url: enc_url.clone(),
				out_file: dec_path.clone(),
				silent: true
			})
		},
		MockStdin("hunter2\n")
	).await;

	assert_eq!(result, std::process::ExitCode::SUCCESS);

	let dec_contents = tokio::fs::read(&dec_path).await.unwrap();

	assert_eq!(contents, dec_contents);

	let result = run_with_io(
		Cli {
			command: Command::Fetch(FetchArgs {
				url: enc_url,
				out_file: dec_path.clone(),
				silent: true
			})
		},
		MockStdin("not_hunter2\n")
	).await;

	assert_eq!(result, std::process::ExitCode::FAILURE);

	let dec_contents_wrong = tokio::fs::read(&dec_path).await.unwrap();
	assert_eq!(dec_contents, dec_contents_wrong);
}

#[tokio::test]
async fn end_to_end_chaff() {
	let tmp = tempfile::tempdir().unwrap();
	let chaff_path = tmp.as_ref().join("chaff.ssec");
	let dec_path = tmp.as_ref().join("unchaff");

	let result = run_with_io(
		Cli {
			command: Command::Chaff(ChaffArgs {
				out_file: chaff_path.clone(),
				size: "30MB".to_string(),
				random_size_max: None,
				silent: true
			})
		},
		EmptyMockStdin
	).await;

	assert_eq!(result, std::process::ExitCode::SUCCESS);

	let result = run_with_io(
		Cli {
			command: Command::Dec(DecArgs {
				in_file: chaff_path,
				out_file: dec_path,
				silent: true
			})
		},
		MockStdin("hunter2\n")
	).await;

	assert_eq!(result, std::process::ExitCode::FAILURE);
}
