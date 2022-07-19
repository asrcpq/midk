use crate::AudioBuffer;

pub struct SampleDb {
	pub release: f32,
	pub keys: Vec<Key>,
}

impl SampleDb {
	pub fn load_config(config_str: &str) -> Self {
		let json: serde_json::Value = serde_json::from_str(config_str).unwrap();
		let mut keys = Vec::new();
		let release = json["release"].as_f64().unwrap_or(0.02) as f32;
		for key in json["keys"].as_array().unwrap().iter() {
			let file = key["file"].as_str().unwrap();
			eprintln!("Load sample: {}", file);
			let mut reader = claxon::FlacReader::open(file).unwrap();
			// force 48khz
			assert_eq!(reader.streaminfo().sample_rate, 48000);
			let mut sample = [Vec::new(), Vec::new()];
			let mut frame_reader = reader.blocks();
			let mut buffer = Vec::new();
			while let Ok(Some(block)) = frame_reader.read_next_or_eof(buffer) {
				let channels = if block.channels() == 2 {
					[0, 1]
				} else {
					[0, 0]
				};
				for channel in 0..2 {
					let block_sample: Vec<f32> = block
						.channel(channels[channel])
						.iter()
						.map(|x| -x as f32 / i32::MIN as f32)
						.collect();
					sample[channel].extend(block_sample);
				}
				buffer = block.into_buffer();
			}
			let note = key["note"].as_i64().unwrap() as u8;
			let velocity = key["velocity"].as_i64().unwrap() as u8;
			keys.push(Key {
				buffer: sample,
				note,
				velocity,
			});
		}
		Self { release, keys }
	}
}

pub struct Key {
	pub buffer: AudioBuffer,
	pub note: u8,
	pub velocity: u8,
}
