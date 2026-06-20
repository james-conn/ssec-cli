#[derive(Debug, Clone, Copy)]
pub enum IoMode {
	Interactive,
	#[cfg(test)]
	TestMockedInput(&'static [u8])
}

impl IoMode {
	#[inline]
	pub fn is_interactive(&self) -> bool {
		matches!(self, IoMode::Interactive)
	}
}
