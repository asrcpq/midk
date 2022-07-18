use ksmp::sample_db::SampleDb;
use ksmp::polyman::Polyman;

fn main() {
	let mut iter = std::env::args();
	iter.next();
	let path: String = iter.next().unwrap();
	let config = std::fs::read_to_string(path).unwrap();
	let sample_db = SampleDb::load_config(&config);
	let mut polyman = Polyman::new(sample_db);
	let (client, _status) = jack::Client::new(
		"midk_ksmp",
		jack::ClientOptions::NO_START_SERVER,
	).unwrap();
	// let sample_rate = client.sample_rate();

	let midi_in = client.register_port("midi_in", jack::MidiIn::default())
		.unwrap();
	let mut audio_out1 = client.register_port("audio_out1", jack::AudioOut::default())
		.unwrap();
	let mut audio_out2 = client.register_port("audio_out2", jack::AudioOut::default())
		.unwrap();

	let callback = move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
		let mut events: Vec<_> = midi_in.iter(ps).collect();
		events.sort_unstable_by_key(|x| x.time);
		let out1 = audio_out1.as_mut_slice(ps);
		let out2 = audio_out2.as_mut_slice(ps);
		let mut event_idx = 0;
		for (idx, (s1, s2)) in out1.iter_mut().zip(out2.iter_mut()).enumerate() {
			// send all events
			loop {
				if event_idx >= events.len() { break }
				let event = events[event_idx];
				if event.time > idx as u32 { break }
				if event.bytes[0] == 154 {
					polyman.keydown(event.bytes[1], event.bytes[2]);
				} else if event.bytes[0] == 138 {
					polyman.keyup(event.bytes[1]);
				}
				event_idx += 1;
			}

			[*s1, *s2] = polyman.get_sample();
			polyman.step();
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
