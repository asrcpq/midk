use crate::synth::Synth;

pub trait SynthGenerator: Send {
	fn set_sr(&mut self, sr: usize);
	fn generate(&self, note: u8, velocity: f32) -> Box<dyn Synth>;
}
