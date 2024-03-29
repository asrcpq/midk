use ksmp::polyman::Polyman;
use ksmp::sample_db::SampleDb;

fn main() {
	let mut iter = std::env::args();
	iter.next();
	let path: String = iter.next().unwrap();
	let config = std::fs::read_to_string(path).unwrap();
	let (client, _status) =
		jack::Client::new("midk_ksmp", jack::ClientOptions::NO_START_SERVER)
			.unwrap();
	assert_eq!(client.sample_rate(), 48000);

	let midi_in = client
		.register_port("midi_in", jack::MidiIn::default())
		.unwrap();
	let mut audio_out1 = client
		.register_port("audio_out1", jack::AudioOut::default())
		.unwrap();
	let mut audio_out2 = client
		.register_port("audio_out2", jack::AudioOut::default())
		.unwrap();
	let sample_db = SampleDb::load_config(&config);
	let mut polyman = Polyman::new(sample_db);

	let callback = move |_: &jack::Client,
	                     ps: &jack::ProcessScope|
	      -> jack::Control {
		let mut events: Vec<_> = midi_in.iter(ps).collect();
		events.sort_unstable_by_key(|x| x.time);
		let out1 = audio_out1.as_mut_slice(ps);
		let out2 = audio_out2.as_mut_slice(ps);
		let mut event_idx = 0;
		for (idx, (s1, s2)) in out1.iter_mut().zip(out2.iter_mut()).enumerate()
		{
			// send all events
			loop {
				if event_idx >= events.len() {
					break;
				}
				let event = events[event_idx];
				if event.time > idx as u32 {
					break;
				}
				if (144..160).contains(&event.bytes[0]) {
					polyman.keydown(event.bytes[1], event.bytes[2]);
				} else if (128..144).contains(&event.bytes[0]) {
					polyman.keyup(event.bytes[1]);
				} else if (176..192).contains(&event.bytes[0]) && event.bytes[1] == 64 {
					if event.bytes[2] == 0 {
						polyman.sustain(false);
					} else if event.bytes[2] == 127 {
						polyman.sustain(true);
					}
				}
				event_idx += 1;
			}

			let [ss1, ss2] = polyman.get_sample();
			*s1 = ss1;
			*s2 = ss2;
			polyman.step();
		}
		jack::Control::Continue
	};

	let active_client = client
		.activate_async((), jack::ClosureProcessHandler::new(callback))
		.unwrap();

	std::thread::park();
	active_client.deactivate().unwrap();
}
