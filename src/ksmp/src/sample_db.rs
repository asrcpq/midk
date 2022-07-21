use crate::AudioBuffer;

pub struct SampleDb {
	pub attack: f32,
	pub release: f32,
	pub keys: Vec<Key>,
}

impl SampleDb {
	pub fn load_config(config_str: &str) -> Self {
		let json: serde_json::Value = serde_json::from_str(config_str).unwrap();
		let mut keys = Vec::new();
		let release = json["release"].as_f64().unwrap_or(0.02) as f32;
		let attack = json["attack"].as_f64().unwrap_or(0.0) as f32;
		let sample_dir = json["sample_dir"].as_str().unwrap();
		for entry in std::fs::read_dir(sample_dir).unwrap() {
			let path = entry.unwrap().path();
			let filename = path.file_name().unwrap();
			let mut v_flag = false;
			let mut note = 0;
			let mut velocity = 0;
			for ch in filename.to_str().unwrap().chars() {
				if ch == 'v' {
					v_flag = true;
				} else if ch.is_digit(10) {
					if v_flag {
						velocity *= 10;
						velocity += ch.to_digit(10).unwrap() as u8;
					} else {
						note *= 10;
						note += ch.to_digit(10).unwrap() as u8;
					}
				} else if ch != 'n' {
					break;
				}
			}
			let mut reader = hound::WavReader::open(path).unwrap();
			let mut sample = [Vec::new(), Vec::new()];
			let mut channel = 0;
			for s in reader.samples::<f32>() {
				sample[channel].push(s.unwrap());
				channel += 1;
				channel %= 2;
			}
			let len0 = sample[0].len();
			let len1 = sample[1].len();
			if len0 < 1000 || len1 < 1000 || len0 != len1 {
				eprintln!("bad sample, skipped");
				continue;
			}
			keys.push(Key {
				buffer: sample,
				note,
				velocity,
			});
			eprintln!("loaded n{}v{}", note, velocity);
		}
		Self {
			release,
			attack,
			keys,
		}
	}
}

pub struct Key {
	pub buffer: AudioBuffer,
	pub note: u8,
	pub velocity: u8,
}
