use rand::{SeedableRng, TryRngCore};
use crate::cli::{Cli, Command, EncArgs, DecArgs};
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
		MockStdin("hunter2\n")
	).await;

	assert_eq!(result, std::process::ExitCode::SUCCESS);

	let result = run_with_io(
		Cli {
			command: Command::Dec(DecArgs {
				in_file: out_path.clone(),
				out_file: dec_path.clone()
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
				in_file: out_path,
				out_file: dec_path
			})
		},
		MockStdin("not_hunter2\n")
	).await;

	assert_eq!(result, std::process::ExitCode::FAILURE);
}
