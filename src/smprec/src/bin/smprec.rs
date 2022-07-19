use std::sync::mpsc::channel;

fn main() {
	let args = aarg::parse();
	let output_dir = args
		.get("output_dir")
		.map(|x| x[0].clone())
		.unwrap_or_else(|| "/tmp/midk_smprec".to_string());
	let _ = std::fs::create_dir(&output_dir);
	let (client, _status) =
		jack::Client::new("midk_smprec", jack::ClientOptions::NO_START_SERVER)
			.unwrap();

	let audio_in1 = client
		.register_port("audio_in1", jack::AudioIn::default())
		.unwrap();
	let audio_in2 = client
		.register_port("audio_in2", jack::AudioIn::default())
		.unwrap();
	let mut midi_out = client
		.register_port("midi_out", jack::MidiOut::default())
		.unwrap();
	let mut input = String::new();
	eprintln!("Connect ports, then press ENTER to start recording");
	std::io::stdin().read_line(&mut input).unwrap();
	let sample_rate = client.sample_rate();
	let spec = hound::WavSpec {
		channels: 2,
		sample_rate: sample_rate as u32,
		bits_per_sample: 32,
		sample_format: hound::SampleFormat::Float,
	};
	let mut writer =
		hound::WavWriter::create(format!("{}/tmp.wav", output_dir), spec)
			.unwrap();

	let trigger = 1e-5;
	let mut trigger_flag = false;
	let mut sender_flag = true;
	let (tx, rx) = channel();
	let callback =
		move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
			if sender_flag {
				let mut writer = midi_out.writer(ps);
				writer
					.write(&jack::RawMidi {
						time: 0,
						bytes: &[154, 69, 100],
					})
					.unwrap();
				sender_flag = false;
			}
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
				} else if s1.abs() <= trigger && s2.abs() <= trigger {
					eprintln!("stop recording");
					tx.send((false, 0.0, 0.0)).unwrap();
					return jack::Control::Quit;
				}
				tx.send((true, s1, s2)).unwrap();
				writer.write_sample(s1).unwrap();
				writer.write_sample(s2).unwrap();
			}
			jack::Control::Continue
		};

	let active_client = client
		.activate_async((), jack::ClosureProcessHandler::new(callback))
		.unwrap();
	let mut counter = 0;
	while let Ok((flag, s1, s2)) = rx.recv() {
		if !flag {
			break;
		}
		counter += 1;
		if counter > 100000 {
			eprintln!("signal: {} {}", s1, s2);
			counter = 0;
		}
	}
	active_client.deactivate().unwrap();
}
