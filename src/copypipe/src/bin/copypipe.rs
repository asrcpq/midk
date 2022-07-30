fn main() {
	let (client, _status) = jack::Client::new(
		"midk_audiorec",
		jack::ClientOptions::NO_START_SERVER,
	)
	.unwrap();

	let audio_in1 = client
		.register_port("audio_in1", jack::AudioIn::default())
		.unwrap();
	let audio_in2 = client
		.register_port("audio_in2", jack::AudioIn::default())
		.unwrap();
	let mut audio_out1 = client
		.register_port("audio_out1", jack::AudioOut::default())
		.unwrap();
	let mut audio_out2 = client
		.register_port("audio_out2", jack::AudioOut::default())
		.unwrap();

	let callback =
		move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
			let in1 = audio_in1.as_slice(ps);
			let in2 = audio_in2.as_slice(ps);
			let out1 = audio_out1.as_mut_slice(ps);
			let out2 = audio_out2.as_mut_slice(ps);
			for (s1, d1) in in1.iter().zip(out1.iter_mut()) {
				*d1 = *s1;
			}
			for (s2, d2) in in2.iter().zip(out2.iter_mut()) {
				*d2 = *s2;
			}
			jack::Control::Continue
		};

	let active_client = client
		.activate_async((), jack::ClosureProcessHandler::new(callback))
		.unwrap();
	std::thread::park();
	active_client.deactivate().unwrap();
}
