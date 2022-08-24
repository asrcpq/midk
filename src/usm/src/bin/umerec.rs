fn main() {
	let path = {
		let mut iter = std::env::args();
		iter.next();
		iter.next().unwrap()
	};
	let (client, _status) = jack::Client::new(
		"midk_umecat",
		jack::ClientOptions::NO_START_SERVER,
	).unwrap();
	let sr = client.sample_rate();
	let bs = client.buffer_size();
	let midi_in = client
		.register_port("midi_in", jack::MidiIn::default())
		.unwrap();
	let callback =
		move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
			jack::Control::Continue
		};
}
