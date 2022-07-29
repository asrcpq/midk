fn main() {
	let args = aarg::parse().unwrap();
	let mut filters = [false; 128];
	for x in args
		.get("--whitelist")
		.unwrap()
		.iter()
	{
		let b = x.parse::<usize>().unwrap();
		filters[b] = true;
	}
	let (client, _status) = jack::Client::new(
		"midk_midifilter",
		jack::ClientOptions::NO_START_SERVER,
	)
	.unwrap();

	let midi_in = client
		.register_port("midi_in", jack::MidiIn::default())
		.unwrap();
	let mut midi_out = client
		.register_port("midi_out", jack::MidiOut::default())
		.unwrap();

	let callback =
		move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
			let mut writer = midi_out.writer(ps);
			for event in midi_in.iter(ps) {
				if event.bytes[0] == 154 || event.bytes[0] == 138 {
					if filters[event.bytes[1] as usize] {
						writer.write(&event).unwrap();
					}
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
