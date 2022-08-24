pub struct Align {
	sr: u32,
	f: u32,
	f_offset: u32,
	t: u32,
}

impl Align {
	pub fn new(sr: u32) -> Self {
		Self {
			sr,
			f: 0,
			f_offset: 0,
			t: 0,
		}
	}

	// timer won't go
	fn get_abs_frame(&self, dt: u32) -> u32 {
		let f = ((
				(self.t + dt) as u64 * self.sr as u64
				+ 500_000
			) / 1_000_000) as u32;
		if f < self.f {
			eprintln!("ERROR, frame exceeds time, should never happen");
			self.f
		} else {
			f
		}
	}

	pub fn get_frame(&self, dt: u32) -> u32 {
		let f = self.get_abs_frame(dt);
		if f < self.f_offset {
			eprintln!("WARN: past frame {} vs {}, dt: {}", f, self.f_offset, dt);
			0
		} else {
			f - self.f_offset
		}
	}

	pub fn go_timer(&mut self, dt: u32) {
		let f = self.get_frame(dt);
		self.t += dt;
		self.f = f;
	}

	pub fn go_frame(&mut self, df: u32) {
		self.f_offset += df;
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_align() {
		let a = Align::new(48000);
		assert_eq!(a.get_frame(999), 47);
	}
}
