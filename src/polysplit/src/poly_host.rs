use crate::synth_generator::SynthGenerator;
use crate::synth::Synth;

use std::collections::HashMap;

pub struct PolyHost {
	generator: Box<dyn SynthGenerator>,
	active: HashMap<u8, Box<dyn Synth>>,
	release: Vec<Box<dyn Synth>>,
}

impl PolyHost {
	pub fn new(generator: Box<dyn SynthGenerator>) -> Self {
		Self {
			generator,
			active: Default::default(),
			release: Default::default(),
		}
	}

	pub fn run(mut self) {
		let (client, _status) = jack::Client::new(
			"midk_polyhost",
			jack::ClientOptions::NO_START_SERVER,
		).unwrap();
		let sample_rate = client.sample_rate();
		self.generator.set_sr(sample_rate);

		let midi_in = client
			.register_port("midi_in", jack::MidiIn::default())
			.unwrap();
		let mut audio_out1 = client
			.register_port("audio_out1", jack::AudioOut::default())
			.unwrap();
		let mut audio_out2 = client
			.register_port("audio_out2", jack::AudioOut::default())
			.unwrap();

		let callback =
			move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
				for event in midi_in.iter(ps) {
					if event.bytes[0] == 154 {
						self.active.insert(event.bytes[1], self.generator.generate(
							event.bytes[1],
							event.bytes[2] as f32 / 128.0,
						));
					} else if event.bytes[0] == 138 {
						// TODO: send release event
						self.active.remove(&event.bytes[1]);
					}
				}
				let out1 = audio_out1.as_mut_slice(ps);
				let out2 = audio_out2.as_mut_slice(ps);
				for v in out1.iter_mut() { *v = 0.0 }
				for v in out2.iter_mut() { *v = 0.0 }
				for (_, synth) in self.active.iter_mut() {
					synth.sample(out1, out2);
				}
				jack::Control::Continue
			};

		let active_client = client
			.activate_async((), jack::ClosureProcessHandler::new(callback))
			.unwrap();
		std::thread::park();
		active_client.deactivate().unwrap();
	}
}
