#[derive(Debug, Clone, Copy, Default)]
pub struct Stats {
	pub propagations: u64,
	pub conflicts: u64,
	pub restarts: u64,
}

impl Stats {
	pub fn print_summary(&self) {
		println!("c propagations: {:9}", self.propagations);
		println!("c    conflicts: {:9}", self.conflicts);
		println!("c     restarts: {:9}", self.restarts);
	}
}
