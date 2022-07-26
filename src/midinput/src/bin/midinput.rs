use uinput::event::keyboard::Key;

fn main() {
	let mut device = uinput::default().unwrap()
		.name("midk_midinput").unwrap()
		.event(uinput::event::Keyboard::All).unwrap()
		.create().unwrap();

	let (client, _status) = jack::Client::new(
		"midk_midinput",
		jack::ClientOptions::NO_START_SERVER,
	)
	.unwrap();

	let midi_in = client
		.register_port("midi_in", jack::MidiIn::default())
		.unwrap();

	let callback =
		move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
			// TODO: time sort
			for event in midi_in.iter(ps) {
				eprintln!("{:?}", event);
				if event.bytes[0] != 154 {
					continue
				}
				let key = match event.bytes[1] {
					24 => Key::_1,
					25 => Key::_2,
					26 => Key::_3,
					_ => Key::Space,
				};
				device.click(&key).unwrap();
				device.synchronize().unwrap();
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
