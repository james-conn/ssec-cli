use rand::{SeedableRng, TryRngCore};
use crate::{GetBufRead, cli::{Cli, Command, EncArgs, DecArgs}, run_with_io};

const RNG_SEED: u64 = 12345678;

struct MockStdin(String);

impl GetBufRead for MockStdin {
	fn get_bufread(&self) -> impl std::io::BufRead {
		self.0.as_bytes()
	}
}

#[tokio::test]
async fn end_to_end() {
	let mut rng = rand::rngs::SmallRng::seed_from_u64(RNG_SEED);
	let tmp = tempfile::tempdir().unwrap();
	let in_path = tmp.as_ref().join("file1");
	let out_path = tmp.as_ref().join("file.ssec");
	let dec_path = tmp.as_ref().join("file2");

	let mut contents = vec![0; 100000];
	rng.try_fill_bytes(&mut contents).unwrap();
	tokio::fs::write(&in_path, &contents).await.unwrap();

	let result = run_with_io(
		Cli {
			command: Command::Enc(EncArgs {
				in_file: in_path,
				out_file: Some(out_path.clone())
			})
		},
		MockStdin(String::from("hunter2\n")),
		std::io::sink()
	).await;

	assert_eq!(result, std::process::ExitCode::SUCCESS);

	let result = run_with_io(
		Cli {
			command: Command::Dec(DecArgs {
				in_file: out_path.clone(),
				out_file: dec_path.clone()
			})
		},
		MockStdin(String::from("hunter2\n")),
		std::io::sink()
	).await;

	assert_eq!(result, std::process::ExitCode::SUCCESS);

	let dec_contents = tokio::fs::read(&dec_path).await.unwrap();

	assert_eq!(contents, dec_contents);

	let result = run_with_io(
		Cli {
			command: Command::Dec(DecArgs {
				in_file: out_path,
				out_file: dec_path
			})
		},
		MockStdin(String::from("not_hunter2\n")),
		std::io::sink()
	).await;

	assert_eq!(result, std::process::ExitCode::FAILURE);
}
