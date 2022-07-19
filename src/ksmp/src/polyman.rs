use std::collections::HashMap;
use std::sync::Arc;

use crate::sample_db::SampleDb;
use crate::AudioBuffer;

type VelocityBuffer = (u8, Arc<AudioBuffer>);
type Buffers = Vec<(u8, Vec<VelocityBuffer>)>;

pub struct Polyman {
	playkeys: HashMap<u8, Playkey>,
	buffers: Buffers,
	release: f32,
	volume: f32,
	sustain: Option<Vec<u8>>,
}

impl Polyman {
	pub fn active_keys(&self) -> usize {
		self.playkeys.len()
	}

	pub fn new(db: SampleDb) -> Self {
		let mut buffers: Buffers = Vec::new();
		for key in db.keys.into_iter() {
			let note = key.note;
			let vbuf = (key.velocity, Arc::new(key.buffer));
			match buffers.iter_mut().find(|x| x.0 == note) {
				None => buffers.push((note, vec![vbuf])),
				Some(x) => x.1.push(vbuf),
			}
		}
		buffers.sort_unstable_by_key(|x| x.0);
		for (_, vbufs) in buffers.iter_mut() {
			vbufs.sort_unstable_by_key(|x| x.0);
		}
		Self {
			playkeys: HashMap::new(),
			buffers,
			release: 1f32 / (db.release * 48000.0),
			volume: 70f32,
			sustain: None,
		}
	}

	pub fn get_sample(&self) -> [f32; 2] {
		let mut result = [0.0; 2];
		for (_, playkey) in self.playkeys.iter() {
			let offset0 = playkey.sample_offset as usize;
			#[allow(clippy::needless_range_loop)]
			for channel in 0..2 {
				let buf = &playkey.buffer[channel];
				if offset0 >= buf.len() - 1 {
					break;
				}
				let s0 = buf[offset0];
				let s1 = buf[offset0 + 1];
				let f = playkey.sample_offset.fract();
				let mut s2 = f * (s1 - s0) + s0;
				if let Some(release) = playkey.release {
					s2 *= release;
				}
				result[channel] += s2 * self.volume;
			}
		}
		result
	}

	pub fn step(&mut self) {
		for (key, mut playkey) in std::mem::take(&mut self.playkeys).into_iter()
		{
			playkey.sample_offset += playkey.step;
			if (playkey.buffer[0].len() as f32) < playkey.sample_offset {
				continue;
			}
			if let Some(ref mut release) = playkey.release {
				*release -= self.release;
				if *release <= 0.0 {
					continue;
				}
			}
			self.playkeys.insert(key, playkey);
		}
	}

	pub fn sustain(&mut self, on: bool) {
		if on {
			self.sustain = Some(Vec::new());
		} else if let Some(notes) = self.sustain.take() {
			for note in notes.into_iter() {
				self.keyup(note);
			}
		} else {
			eprintln!("ERROR: sustain off, but never on!");
		}
	}

	pub fn keydown(&mut self, note: u8, velocity: u8) {
		if let Some(ref mut notes) = self.sustain {
			notes.retain(|&x| x != note);
		}
		let (sample_note, vbufs) =
			match self.buffers.iter().enumerate().find(|(_, x)| x.0 > note) {
				None => self.buffers.last().unwrap(),
				Some((idx, _)) => {
					if idx == 0 {
						&self.buffers[0]
					} else {
						&self.buffers[idx - 1]
					}
				}
			};
		let (sample_velocity, buffer) =
			match vbufs.iter().enumerate().find(|(_, x)| x.0 > velocity) {
				None => &vbufs.last().unwrap(),
				Some((idx, _)) => {
					if idx == 0 {
						&vbufs[0]
					} else {
						&vbufs[idx - 1]
					}
				}
			};
		let step = 2f32.powf((note as f32 - *sample_note as f32) / 12.0);
		eprintln!("note: {} vel: {} step: {}", sample_note, sample_velocity, step);
		let playkey = Playkey {
			buffer: buffer.clone(),
			sample_offset: 0.0,
			step,
			release: None,
		};
		self.playkeys.insert(note, playkey);
	}

	pub fn keyup(&mut self, note: u8) {
		if let Some(ref mut notes) = self.sustain {
			notes.push(note);
			// TODO: key pressed again?
			return;
		}
		if let Some(mut playkey) = self.playkeys.get_mut(&note) {
			playkey.release = Some(1.0);
		} else {
			eprintln!("key up, but not found in playkeys");
		}
	}
}

struct Playkey {
	// NOTE: limitation: audiobuffer framerate must be the same as playback
	buffer: Arc<AudioBuffer>,
	sample_offset: f32,
	// delta time applied to sample_offset for each frame
	step: f32,
	release: Option<f32>,
}
