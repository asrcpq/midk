use midk_polysplit::poly_host::PolyHost;
use midk_polysplit::synth::Synth;
use midk_polysplit::synth_generator::SynthGenerator;

use midk_synthwidget::seg::{Seg, SegPredefined};
use midk_synthwidget::filter::SimpleIir;

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
		let speed = freq * self.frame_t;
		let sws = SwSynth {
			osc1: Seg::new_predefined(
				//SegPredefined::Sine8Points,
				//SegPredefined::Pulse(0.3),
				SegPredefined::Saw,
				speed,
			),
			osc2: Seg::new(
				vec![
					(0., 1.),
					(0.15, 1.),
					(0.15, 0.),
					(0.4719, 0.),
					(0.4719, 1.),
					(0.6219, 1.),
					(0.6219, 0.),
					(0.9439, 0.),
					(0.9439, 1.),
					(1., 1.),
				],
				speed,
			).set_loop(1.0, -1),
			amp: Seg::new(
				vec![
					(0., 1.0),
					(0.3, 0.25),
					(3., 0.),
				],
				self.frame_t,
			),
			iir: SimpleIir::new(0.1),
			release: None,
			level: velocity / 5.0,
			buffer: Vec::new(),
			frame_t: self.frame_t,
		};
		Box::new(sws)
	}
}

// provide a simple example
struct SwSynth {
	osc1: Seg,
	osc2: Seg,
	amp: Seg,
	iir: SimpleIir,
	release: Option<Seg>,
	level: f32,
	buffer: Vec<f32>,
	frame_t: f32,
}

impl Synth for SwSynth {
	fn set_end(&mut self, _smp_count: usize) {
		// TODO: precise timing
		self.release = Some(Seg::new(
			vec![
				(0., 1.),
				(0.5, 0.),
			],
			self.frame_t,
		))
	}

	fn sample(&mut self, data_l: &mut [f32], data_r: &mut [f32]) -> Option<usize> {
		self.buffer.resize(data_l.len(), 0.0);
		self.osc1.write(&mut self.buffer, |x, y| {*x = 0.75 * y});
		self.osc2.write(&mut self.buffer, |x, y| {*x += y});
		self.amp.write(&mut self.buffer, |x, y| {*x *= y});
		self.iir.write(&mut self.buffer, |x, y| {*x = y});
		if let Some(rel) =self.release.as_mut() {
			if let Some(t) = rel.write(&mut self.buffer, |x, y| {*x *= y}) {
				return Some(t);
			}
		}
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
