pub struct SimpleIir {
	k: f32,
	prev: f32,
}

impl SimpleIir {
	pub fn new(k: f32) -> Self {
		Self {
			k,
			prev: 0.0,
		}
	}

	pub fn reset(&mut self) {
		self.prev = 0.;
	}

	// F is blending function
	pub fn write<F>(&mut self, buffer: &mut [f32], f_blend: F)
		where F: Fn(&mut f32, f32)
	{
		for s in buffer.iter_mut() {
			let s2 = *s * self.k + self.prev * (1.0 - self.k);
			self.prev = s2;
			f_blend(s, s2);
		}
	}
}
