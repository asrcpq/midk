use std::sync::mpsc::channel;

fn main() {
	let args = aarg::parse();
	let output_dir = args
		.get("output_dir")
		.map(|x| x[0].clone())
		.unwrap_or_else(|| "/tmp/midk_smprec".to_string());
	std::fs::create_dir(&output_dir).unwrap();
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
	let mut wav_writer =
		hound::WavWriter::create(format!("{}/tmp.wav", output_dir), spec)
			.unwrap();

	let trigger = 3e-4;
	let mut trigger_flag = false;
	let mut sender_flag = true;
	// let note_table = [
	// 	21,
	// 	24, 28, 33,
	// 	36, 40, 45,
	// 	48, 52, 57,
	// 	60, 64, 69,
	// 	72, 76, 81,
	// 	84, 88, 93,
	// 	96, 100, 105,
	// 	108,
	// ];
	// let velocity_table = [
	// 	10, 20, 30,
	// 	40, 50, 60,
	// 	70, 75, 80, 85,
	// 	90, 95, 100, 105,
	// 	110, 120,
	// ];
	let note_table = [24, 36, 48, 60, 72, 84, 96];
	let velocity_table = [20, 50, 80, 110];
	let mut low_counter = 0;
	let mut nid = 0;
	let mut vid = 0;
	let (tx, rx) = channel();
	let callback =
		move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
			let note = note_table[nid];
			let velocity = velocity_table[vid];
			let mut writer = midi_out.writer(ps);
			if sender_flag {
				wav_writer = hound::WavWriter::create(format!(
					"{}/n{}v{}.wav",
					output_dir,
					note,
					velocity,
				), spec).unwrap();
				writer
					.write(&jack::RawMidi {
						time: 0,
						bytes: &[154, note, velocity],
					})
					.unwrap();
				sender_flag = false;
			}
			let in1 = audio_in1.as_slice(ps);
			let in2 = audio_in2.as_slice(ps);
			for (&s1, &s2) in in1.iter().zip(in2.iter()) {
				if !trigger_flag {
					if s1.abs() > trigger || s2.abs() > trigger {
						eprintln!("start recording {} {}", note, velocity);
						trigger_flag = true;
					} else {
						continue;
					}
				}
				if s1.abs() <= trigger && s2.abs() <= trigger {
					low_counter += 1;
					if low_counter > 24000 {
						low_counter = 0;
						eprintln!("finished");
						vid += 1;
						if vid == velocity_table.len() {
							nid += 1;
							if nid == note_table.len() {
								tx.send(false).unwrap();
								return jack::Control::Quit;
							}
							vid = 0;
						}
						eprintln!("{} {}", nid, vid);
						writer.write(&jack::RawMidi {
							time: 0,
							bytes: &[138, note, velocity],
						})
						.unwrap();
						trigger_flag = false;
						sender_flag = true;
					}
				} else {
					low_counter = 0;
				}
				wav_writer.write_sample(s1).unwrap();
				wav_writer.write_sample(s2).unwrap();
			}
			jack::Control::Continue
		};

	let active_client = client
		.activate_async((), jack::ClosureProcessHandler::new(callback))
		.unwrap();
	while let Ok(true) = rx.recv() {}
	active_client.deactivate().unwrap();
}
