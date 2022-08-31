use std::io::BufRead;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
struct Config {
	pub trigger: f32,
	pub ratio: f32,
	pub gain: f32,
	pub release: f32,
	pub rate: f32,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			trigger: 0.1,
			ratio: 0.7,
			gain: 5.0,
			release: 0.2,
			rate: 0.01,
		}
	}
}

impl Config {
	pub fn set_value(&mut self, key: &str, value: f32) {
		// FIXME: rate and release are not actually updated,
		// because value actually used are
		// release and wet_change_per_sample
		eprintln!("DEBUG: set {} to {}", key, value);
		match key {
			"trigger" => self.trigger = value,
			"ratio" => self.ratio = value,
			"gain" => self.gain = value,
			"release" => self.release = value,
			"rate" => self.rate = value,
			_ => eprintln!("Skip key {}", key),
		}
	}
}

fn main() {
	let args = aarg::parse().unwrap();
	let mut config = Config::default();
	for (k, vs) in args.iter() {
		let k = if let Some(k) = k.strip_prefix("--") {
			k
		} else {
			continue
		};
		let v = if let Some(v) = vs.get(0) {
			v
		} else {
			continue
		};
		let v = if let Ok(v) = v.parse::<f32>() {
			v
		} else {
			continue
		};
		config.set_value(k, v);
	}
	let mut wet = 0.0;
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
	let release = (config.release * sample_rate as f32) as i32;
	let wet_change_per_sample = 1.0 / config.rate / sample_rate as f32;
	let mut samples: i32 = -1;
	let config = Arc::new(Mutex::new(config));
	let config2 = config.clone();

	let callback =
		move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
			let config = config2.lock().unwrap();
			let in1 = audio_in1.as_slice(ps);
			let in2 = audio_in2.as_slice(ps);
			let out1 = audio_out1.as_mut_slice(ps);
			let out2 = audio_out2.as_mut_slice(ps);
			for (i1, i2, o1, o2) in itertools::izip!(in1, in2, out1, out2) {
				let i1 = *i1;
				let i2 = *i2;
				if i1 > config.trigger || i2 > config.trigger {
					samples = 0;
				}
				if samples < 0 {
					wet -= wet_change_per_sample;
					wet = wet.max(0.); 
				} else {
					wet += wet_change_per_sample;
					wet = wet.min(1.); 
				}
				let o1_dry = i1;
				let o2_dry = i2;
				let o1_wet = config.trigger +
					(i1 - config.trigger) * config.ratio;
				let o2_wet = config.trigger +
					(i2 - config.trigger) * config.ratio;
				*o1 = config.gain *
					(o1_wet * wet + o1_dry * (1.0 - wet));
				*o2 = config.gain *
					(o2_wet * wet + o2_dry * (1.0 - wet));
			}
			if samples >= 0 {
				samples += buffer_size as i32;
				if samples > release {
					samples = -1;
				}
			}
			jack::Control::Continue
		};

	let active_client = client
		.activate_async((), jack::ClosureProcessHandler::new(callback))
		.unwrap();
	let stdin = std::io::stdin();
	for line in stdin.lock().lines() {
		let line = line.unwrap();
		let args: Vec<_> = line.split_whitespace().collect();
		if args.len() < 2 { continue }
		let v = if let Ok(v) = args[1].parse::<f32>() {
			v
		} else {
			continue
		};
		let mut config = config.lock().unwrap();
		config.set_value(args[0], v);
	}
	active_client.deactivate().unwrap();
}
