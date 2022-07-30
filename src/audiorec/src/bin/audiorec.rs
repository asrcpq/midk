fn main() {
	let args = aarg::parse().unwrap();
	let trigger = args
		.get("--trigger")
		.map(|x| x[0].parse::<f32>().unwrap())
		.unwrap_or(-0.1);
	let duration = args
		.get("--duration")
		.map(|x| x[0].parse::<f32>().unwrap())
		.unwrap_or(f32::INFINITY);
	let output = args
		.get("--output")
		.map(|x| x[0].clone())
		.unwrap_or_else(|| "/tmp/midk_audiorec.wav".to_string());
	let mut trigger_flag = false;
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
	let sample_rate = client.sample_rate();
	let mut total_samples = 0;

	let spec = hound::WavSpec {
		channels: 2,
		sample_rate: sample_rate as u32,
		bits_per_sample: 32,
		sample_format: hound::SampleFormat::Float,
	};
	let mut writer = hound::WavWriter::create(output, spec).unwrap();

	let callback =
		move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
			let in1 = audio_in1.as_slice(ps);
			let in2 = audio_in2.as_slice(ps);
			for (&s1, &s2) in in1.iter().zip(in2.iter()) {
				if !trigger_flag {
					if s1.abs() > trigger || s2.abs() > trigger {
						eprintln!("start recording");
						trigger_flag = true;
					} else {
						continue;
					}
				}
				writer.write_sample(s1).unwrap();
				writer.write_sample(s2).unwrap();
			}
			total_samples += in1.len();
			if total_samples as f32 / sample_rate as f32 > duration {
				jack::Control::Quit
			} else {
				jack::Control::Continue
			}
		};

	let active_client = client
		.activate_async((), jack::ClosureProcessHandler::new(callback))
		.unwrap();
	let mut line = String::new();
	std::io::stdin().read_line(&mut line).unwrap();
	active_client.deactivate().unwrap();
}
