use std::collections::HashMap;

fn main() {
	let (client, _status) = jack::Client::new(
		"midk_toysynth",
		jack::ClientOptions::NO_START_SERVER,
	).unwrap();
	let sample_rate = client.sample_rate();
	let frame_t = 1.0 / sample_rate as f32;
	let mut note_on = HashMap::new();

	let midi_in = client.register_port("midi_in", jack::MidiIn::default())
		.unwrap();
	let mut audio_out1 = client.register_port("audio_out1", jack::AudioOut::default())
		.unwrap();
	let mut audio_out2 = client.register_port("audio_out2", jack::AudioOut::default())
		.unwrap();

	let callback = move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
		for event in midi_in.iter(ps) {
			if event.bytes[0] == 154 {
				note_on.insert(event.bytes[1], (event.bytes[2], 0f32));
			} else if event.bytes[0] == 138 {
				note_on.remove(&event.bytes[1]);
			}
		}
		let out1 = audio_out1.as_mut_slice(ps);
		let out2 = audio_out2.as_mut_slice(ps);
		for (v1, v2) in out1.iter_mut().zip(out2.iter_mut()) {
			*v1 = 0.0;
			*v2 = 0.0;
			for (&note, value) in note_on.iter_mut() {
				let y = &mut value.1;
				let vel = value.0;
				*y += 440.0 * 2f32.powf((note as i32 - 57) as f32 / 12.0) * frame_t;
				if *y >= 1.0 { *y -= 2.0 }
				let k = 3.0;
				*v1 += (*y >= 0.0) as u32 as f32 * vel as f32 / 128.0 / k;
				*v2 += (*y >= 0.0) as u32 as f32 * vel as f32 / 128.0 / k;
			}
		}
		jack::Control::Continue
	};

	let active_client = client
		.activate_async((), jack::ClosureProcessHandler::new(callback))
		.unwrap();
	let mut line = String::new();
	std::io::stdin().read_line(&mut line).unwrap();
	active_client.deactivate().unwrap();
}
