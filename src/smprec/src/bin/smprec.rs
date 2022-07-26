use std::sync::mpsc::channel;

fn main() {
	let args = aarg::parse();
	let output_dir = args
		.get("output_dir")
		.map(|x| x[0].clone())
		.unwrap_or_else(|| "/tmp/midk_smprec".to_string());
	let notes: Vec<u8> = args
		.get("notes")
		.unwrap()
		.into_iter()
		.map(|x| x.parse::<u8>().unwrap())
		.collect();
	let velocities: Vec<u8> = args
		.get("velocities")
		.unwrap()
		.into_iter()
		.map(|x| x.parse::<u8>().unwrap())
		.collect();
	let delay = args.get("delay").map(|x| x[0].parse::<f32>().unwrap());
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
	if let Some(delay) = delay {
		eprintln!("Delay for port connection");
		std::thread::sleep(std::time::Duration::from_secs_f32(delay));
		eprintln!("Delay finished");
	} else {
		let mut input = String::new();
		eprintln!("Connect ports, then press ENTER to start recording");
		std::io::stdin().read_line(&mut input).unwrap();
	}
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

	let down_trigger = 1e-8;
	let up_trigger = 1e-4;
	let mut trigger_state = 0; // 0: wait up, 1: triggered, 2: wait down
	let mut sender_flag = true;
	let mut trigger_counter = 0;
	let mut sample_count = 0;
	let mut nid = 0;
	let mut vid = 0;
	let (tx, rx) = channel();
	let callback = move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
		let note = notes[nid];
		let velocity = velocities[vid];
		let mut writer = midi_out.writer(ps);
		if sender_flag {
			wav_writer = hound::WavWriter::create(
				format!("{}/n{}v{}.wav", output_dir, note, velocity,),
				spec,
			)
			.unwrap();
			eprintln!("send {} {}", note, velocity);
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
			match trigger_state {
				0 => {
					if s1.abs() > up_trigger || s2.abs() > up_trigger {
						eprintln!("start recording {} {}", note, velocity);
						trigger_state = 1;
					} else {
						continue;
					}
				}
				2 => {
					if s1.abs() <= up_trigger && s2.abs() <= up_trigger {
						trigger_counter += 1;
						if trigger_counter > 24000 {
							trigger_counter = 0;
							eprintln!("state 2 finished");
							trigger_state = 0;
							sample_count = 0;
						}
					} else {
						continue;
					}
				}
				_ => {
					if s1.abs() <= down_trigger && s2.abs() <= down_trigger {
						trigger_counter += 1;
						if trigger_counter > 24000 {
							trigger_counter = 0;
							eprintln!(
								"finished {}",
								sample_count as f32 / sample_rate as f32
							);
							vid += 1;
							if vid == velocities.len() {
								nid += 1;
								if nid == notes.len() {
									tx.send(false).unwrap();
									return jack::Control::Quit;
								}
								vid = 0;
							}
							eprintln!("{} {}", nid, vid);
							writer
								.write(&jack::RawMidi {
									time: 0,
									bytes: &[138, note, velocity],
								})
								.unwrap();
							trigger_state = 2;
							sender_flag = true;
						}
					} else {
						trigger_counter = 0;
					}
					wav_writer.write_sample(s1).unwrap();
					wav_writer.write_sample(s2).unwrap();
					sample_count += 1;
				}
			}
		}
		jack::Control::Continue
	};

	let active_client = client
		.activate_async((), jack::ClosureProcessHandler::new(callback))
		.unwrap();
	while let Ok(true) = rx.recv() {}
	active_client.deactivate().unwrap();
}
