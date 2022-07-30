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
			end_count: None,
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
	end_count: Option<usize>,
	cycle: f32,
	t: f32,
	dt: f32, // per frame
	level: f32,
}

impl Synth for Exsquare {
	fn set_end(&mut self, smp_count: usize) {
		self.end_count = Some(smp_count);
	}

	fn sample(&mut self, data_l: &mut [f32], data_r: &mut [f32]) -> Option<usize> {
		// no strict timing, stop at next sample window
		match self.end_count.as_mut() {
			None => {},
			Some(0) => return Some(0),
			Some(x) => *x = x.saturating_sub(data_l.len()),
		}
		for (l, r) in data_l.iter_mut().zip(data_r.iter_mut()) {
			self.t += self.dt;
			if self.t >= self.cycle {
				self.t -= self.cycle;
			}
			if self.t > self.cycle / 2.0 {
				*l += self.level;
				*r += self.level;
			} else {
				*l += -self.level;
				*r += -self.level;
			}
		}
		None
	}
}
