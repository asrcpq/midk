fn main() {
	let args = aarg::parse().unwrap();
	let trigger = args
		.get("--trigger")
		.map(|x| x[0].parse::<f32>().unwrap())
		.unwrap_or(0.3);
	let ratio = args
		.get("--ratio")
		.map(|x| x[0].parse::<f32>().unwrap())
		.unwrap_or(0.3);
	let gain = args
		.get("--gain")
		.map(|x| x[0].parse::<f32>().unwrap())
		.unwrap_or(2.0);
	let release = args
		.get("--release")
		.map(|x| x[0].parse::<f32>().unwrap())
		.unwrap_or(1.0);
	let (client, _status) = jack::Client::new(
		"midk_toycmp",
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
	let sample_rate = client.sample_rate();
	let buffer_size = client.buffer_size();
	let release = (release * sample_rate as f32) as i32;
	let mut samples: i32 = -1;

	let callback =
		move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
			let in1 = audio_in1.as_slice(ps);
			let in2 = audio_in2.as_slice(ps);
			let out1 = audio_out1.as_mut_slice(ps);
			let out2 = audio_out2.as_mut_slice(ps);
			for (i1, i2, o1, o2) in itertools::izip!(in1, in2, out1, out2) {
				let i1 = *i1;
				let i2 = *i2;
				if i1 > trigger || i2 > trigger {
					samples = 0;
				}
				if samples < 0 {
					*o1 = i1;
					*o2 = i2;
				} else {
					*o1 = trigger + (i1 - trigger) * ratio;
					*o2 = trigger + (i2 - trigger) * ratio;
				}
				*o1 *= gain;
				*o2 *= gain;
			}
			samples += buffer_size as i32;
			if samples > release {
				samples = -1;
			}
			jack::Control::Continue
		};

	let active_client = client
		.activate_async((), jack::ClosureProcessHandler::new(callback))
		.unwrap();
	std::thread::park();
	active_client.deactivate().unwrap();
}
