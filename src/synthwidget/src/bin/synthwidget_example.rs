use midk_polysplit::poly_host::PolyHost;
use midk_polysplit::synth::Synth;
use midk_polysplit::synth_generator::SynthGenerator;

use midk_synthwidget::seg::{Seg, SegPredefined};

#[derive(Default)]
pub struct SwGenerator {
	frame_t: f32,
}

impl SynthGenerator for SwGenerator {
	fn set_sr(&mut self, sr: usize) {
		self.frame_t = 1.0 / sr as f32;
	}

	fn generate(&self, note: u8, velocity: f32) -> Box<dyn Synth> {
		let freq = 440.0 * 2f32.powf((note as i32 - 57) as f32 / 12.0);
		let frame_k = freq * self.frame_t;
		let sws = SwSynth {
			osc: Seg::new_predefined(
				SegPredefined::Sine8Points,
				//SegPredefined::Pulse(0.3),
				//SegPredefined::Saw,
				frame_k,
			),
			amp: Seg::new(
				vec![
					(0., 1.0),
					(0.67, 0.25),
				],
				self.frame_t,
			),
			ending_flag: false,
			level: velocity / 5.0,
			buffer: Vec::new(),
		};
		Box::new(sws)
	}
}

// provide a simple example
struct SwSynth {
	osc: Seg,
	amp: Seg,
	ending_flag: bool,
	level: f32,
	buffer: Vec<f32>,
}

impl Synth for SwSynth {
	fn set_end(&mut self, _smp_count: usize) {
		self.ending_flag = true;
	}

	fn sample(&mut self, data_l: &mut [f32], data_r: &mut [f32]) -> Option<usize> {
		self.buffer.resize(data_l.len(), 0.0);
		if self.ending_flag {
			return Some(0)
		}
		self.osc.write(&mut self.buffer, |x, y| {*x = y});
		self.amp.write(&mut self.buffer, |x, y| {*x *= y});
		for (s, v) in data_l.iter_mut().zip(self.buffer.iter()) {
			*s += self.level * v;
		}
		for (s, v) in data_r.iter_mut().zip(self.buffer.iter()) {
			*s += self.level * v;
		}
		None
	}
}

fn main() {
	let ph = PolyHost::new(Box::new(SwGenerator::default()));
	ph.run();
}
