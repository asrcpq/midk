fn main() {
	let (client, _status) = jack::Client::new(
		"midk_mididump",
		jack::ClientOptions::NO_START_SERVER,
	).unwrap();

	let midi_in = client.register_port("midi_in", jack::MidiIn::default())
		.unwrap();

	let callback = move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
		for event in midi_in.iter(ps) {
			eprintln!("{:?}", event);
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
