pub trait Synth: Send {
	// after smp_count samples, the key will be up
	// will not be called more than once
	fn set_end(&mut self, smp_count: usize);

	fn note(&mut self, freq: f32, velocity: f32);

	// data contains N frames, the synth state will also go forward for N frames
	// return Some(frame) if finished in N frame(so it is frame perfect)
	fn sample(&mut self, data_l: &mut [f32], data_r: &mut [f32]) -> bool;
}
