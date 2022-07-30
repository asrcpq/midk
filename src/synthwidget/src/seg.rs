pub struct Seg {
	points: Vec<(f32, f32)>,
	time: f32,
	end_point: f32,
	loop_cd: i32, // loop count down, -1 means infinite
	idx: usize,
	speed: f32, // freq * frame_t
}

#[derive(Clone, Copy, Debug)]
pub enum SegPredefined {
	Sine8Points,
	Pulse(f32), // 0.5 = square
	Triangle,
	Saw,
}

impl Seg {
	pub fn new(points: Vec<(f32, f32)>, speed: f32) -> Self {
		Self {
			points,
			time: 0.,
			end_point: f32::INFINITY,
			loop_cd: 1,
			idx: 0,
			speed,
		}
	}

	pub fn new_predefined(p: SegPredefined, speed: f32) -> Self {
		use SegPredefined::*;
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
			end_point: 1.0,
			loop_cd: -1,
			idx: 0,
			speed,
		}
	}

	// return ending point for non-loop
	pub fn write<F>(&mut self, buffer: &mut [f32], f: F) -> Option<usize>
		where F: Fn(&mut f32, f32)
	{
		if self.loop_cd == 0 { return Some(0) }
		for (idx, s) in buffer.iter_mut().enumerate() {
			let mut next_idx;
			loop {
				next_idx = self.idx + 1;
				if next_idx >= self.points.len() {
					let (_x1, y1) = self.points[self.idx];
					f(s, y1);
					break
				}
				if self.time <= self.points[next_idx].0 {
					let (x1, y1) = self.points[self.idx];
					let (x2, y2) = self.points[next_idx];
					let x3 = self.time;
					let a = x3 - x1;
					let b = x2 - x3;
					f(s, (y1 * b + y2 * a) / (a + b));
					break
				}
				self.idx = next_idx;
			}
			self.time += self.speed;
			if self.time >= self.end_point {
				self.loop_cd -= 1;
				if self.loop_cd == 0 { return Some(idx) }
				self.idx = 0;
				self.time -= self.end_point;
			}
		}
		None
	}
}
