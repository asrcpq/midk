use crate::synth::Synth;
use crate::synth_generator::SynthGenerator;

#[derive(Default)]
pub struct ExsquareGenerator {
	frame_t: f32,
}

impl SynthGenerator for ExsquareGenerator {
	fn set_sr(&mut self, sr: usize) {
		self.frame_t = 1.0 / sr as f32;
	}

	fn generate(&self, note: u8, velocity: f32) -> Box<dyn Synth> {
		let freq = 440.0
			* 2f32.powf((note as i32 - 57) as f32 / 12.0);
		let cycle = 1.0 / freq;
		let exs = Exsquare {
			ending_flag: false,
			end_count: 1,
			release: 1.0,
			release_t: 0.1,

			cycle,
			t: 0.0,
			dt: self.frame_t,
			level: velocity / 5.0,
		};
		Box::new(exs)
	}
}

// provide a simple example
struct Exsquare {
	ending_flag: bool,
	end_count: usize,
	release: f32,
	release_t: f32,

	cycle: f32,
	t: f32,
	dt: f32, // per frame
	level: f32,
}

impl Synth for Exsquare {
	fn set_end(&mut self, smp_count: usize) {
		self.ending_flag = true;
		self.end_count = smp_count;
	}

	fn sample(&mut self, data_l: &mut [f32], data_r: &mut [f32]) -> Option<usize> {
		for (idx, (l, r)) in data_l.iter_mut().zip(data_r.iter_mut()).enumerate() {
			if self.end_count == 0 {
				self.release -= self.dt / self.release_t;
				if self.release <= 0.0 {
					return Some(idx)
				}
			} else if self.ending_flag {
				self.end_count -= 1;
			}
			self.t += self.dt;
			if self.t >= self.cycle {
				self.t -= self.cycle;
			}
			let level = if self.t > self.cycle / 2.0 {
				self.level
			} else {
				-self.level
			};
			*l += level * self.release;
			*r += level * self.release;
		}
		None
	}
}
