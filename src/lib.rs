mod file;

pub mod cli;
pub mod enc;
pub mod dec;

const BAR_STYLE: &str = "[{elapsed_precise}] {binary_bytes_per_sec} {bar} {binary_bytes}/{binary_total_bytes} ({eta})";
