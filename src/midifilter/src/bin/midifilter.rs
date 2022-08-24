fn main() {
	let args = aarg::parse().unwrap();
	let filters = if let Some(whitelist) = args.get("--whitelist") {
		let mut filters = [false; 128];
		for x in whitelist.iter() {
			let b = x.parse::<usize>().unwrap();
			filters[b] = true;
		}
		filters
	} else if let Some(blacklist) = args.get("--blacklist") {
		let mut filters = [true; 128];
		for x in blacklist.iter() {
			let b = x.parse::<usize>().unwrap();
			filters[b] = false;
		}
		filters
	} else {
		// all pass
		[true; 128]
	};
	let (client, _status) = jack::Client::new(
		"midk_midifilter",
		jack::ClientOptions::NO_START_SERVER,
	)
	.unwrap();

	let midi_in = client
		.register_port("midi_in", jack::MidiIn::default())
		.unwrap();
	let mut midi_out1 = client
		.register_port("midi_out1", jack::MidiOut::default())
		.unwrap();
	let mut midi_out2 = client
		.register_port("midi_out2", jack::MidiOut::default())
		.unwrap();

	let callback =
		move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
			let mut writer1 = midi_out1.writer(ps);
			let mut writer2 = midi_out2.writer(ps);
			for event in midi_in.iter(ps) {
				if (128..160).contains(&event.bytes[0]) {
					if filters[event.bytes[1] as usize] {
						writer1.write(&event).unwrap();
					} else {
						writer2.write(&event).unwrap();
					}
				} else {
					writer1.write(&event).unwrap();
					writer2.write(&event).unwrap();
				}
			}
			jack::Control::Continue
		};

	let active_client = client
		.activate_async((), jack::ClosureProcessHandler::new(callback))
		.unwrap();
	std::thread::park();
	active_client.deactivate().unwrap();
}
