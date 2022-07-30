pub struct Segs {
	// wave control points
	// must have 0.0 and 1.0 defined
	points: Vec<(f32, f32)>,
	time: f32,
	idx: usize,
	frame_k: f32, // freq * frame_t
}

#[derive(Clone, Copy, Debug)]
pub enum SegsPredefined {
	Sine8Points,
	Pulse(f32), // 0.5 = square
	Triangle,
	Saw,
}

impl Segs {
	pub fn new_predefined(p: SegsPredefined, frame_k: f32) -> Self {
		use SegsPredefined::*;
		let points = match p {
			Sine8Points => vec![
				(0., 0.),
				(0.125, 0.707),
				(0.25, 1.),
				(0.375, 0.707),
				(0.5, 0.),
				(0.625, -0.707),
				(0.75, -1.),
				(0.875, -0.707),
				(1., 0.),
			],
			Pulse(r) => vec![
				(0., 1.),
				(r, 1.),
				(r, 0.),
				(1.0, 0.),
			],
			Triangle => vec![
				(0., 0.),
				(0.25, 1.),
				(0.75, -1.),
				(1.0, 0.),
			],
			Saw => vec![
				(0., -1.),
				(1.0, 1.),
			],
		};
		Self {
			points,
			time: 0.,
			idx: 0,
			frame_k,
		}
	}

	pub fn write(&mut self, buffer: &mut [f32]) {
		for s in buffer.iter_mut() {
			let mut next_idx;
			loop {
				next_idx = self.idx + 1;
				if next_idx >= self.points.len() {
					self.idx = 0;
					next_idx = 1;
				}
				if self.time >= self.points[self.idx].0 &&
					self.time <= self.points[next_idx].0
				{
					break
				}
				self.idx = next_idx;
			}
			let (x1, y1) = self.points[self.idx];
			let (x2, y2) = self.points[next_idx];
			let x3 = self.time;
			let a = x3 - x1;
			let b = x2 - x3;
			*s = (y1 * b + y2 * a) / (a + b);
			self.time += self.frame_k;
			if self.time >= 1.0 {
				self.time -= 1.0;
			}
		}
	}
}
