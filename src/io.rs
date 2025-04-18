pub trait IoBundle: Send + 'static {
	type IoRead: std::io::BufRead;
	type IoWrite: std::io::Write;

	/// if this is `true` then all other methods are `unimplemented!()` and will panic if called
	fn is_interactive() -> bool {
		false
	}

	fn get_bufread(&self) -> Self::IoRead;
	fn get_write(&self) -> Self::IoWrite;
}

pub struct InteractiveIo;

impl IoBundle for InteractiveIo {
	type IoRead = std::io::Empty;
	type IoWrite = std::io::Sink;

	fn is_interactive() -> bool {
		true
	}

	fn get_bufread(&self) -> Self::IoRead {
		unimplemented!()
	}

	fn get_write(&self) -> Self::IoWrite {
		unimplemented!()
	}
}
