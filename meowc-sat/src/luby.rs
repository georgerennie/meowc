/// https://oeis.org/A182105
#[derive(Clone)]
pub struct Luby {
	un: u64,
	vn: u64,
}

impl Default for Luby {
	fn default() -> Self {
		Self { un: 1, vn: 1 }
	}
}

impl Luby {
	pub fn next(&mut self) -> u64 {
		let v = self.vn;

		if (self.un & self.un.wrapping_neg()) == self.vn {
			self.un += 1;
			self.vn = 1;
		} else {
			self.vn = self.vn << 1;
		}

		v
	}
}
