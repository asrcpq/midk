use std::collections::HashMap;
use std::sync::Arc;

use crate::sample_db::SampleDb;
use crate::AudioBuffer;

type VelocityBuffer = (u8, Arc<AudioBuffer>);
type Buffers = Vec<(u8, Vec<VelocityBuffer>)>;

pub struct Polyman {
	playkeys: HashMap<u8, Playkey>,
	buffers: Buffers,
}

impl Polyman {
	pub fn new(db: SampleDb) -> Self {
		let mut buffers: Buffers = Vec::new();
		for key in db.keys.into_iter() {
			let note = key.note;
			let vbuf = (
				key.velocity,
				Arc::new(key.buffer),
			);
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
		}
	}

	pub fn get_sample(&self) -> [f32; 2] {
		let mut result = [0.0; 2];
		for (_, playkey) in self.playkeys.iter() {
			let offset0 = playkey.sample_offset as usize;
			for channel in 0..2 {
				let s0 = playkey.buffer[channel][offset0];
				let s1 = playkey.buffer[channel][offset0 + 1];
				let f = playkey.sample_offset.fract();
				let mut s2 = f * (s1 - s0) + s0;
				if let Some(release) = playkey.release {
					s2 *= release;
				}
				result[channel] += s2;
			}
		}
		[0.0; 2]
	}

	pub fn step(&mut self) {
		for (key, mut playkey) in std::mem::take(&mut self.playkeys).into_iter() {
			playkey.sample_offset += playkey.step;
			if (playkey.buffer.len() as f32) < playkey.sample_offset {
				continue
			}
			if let Some(mut release) = playkey.release {
				// TODO: remove magic number
				release -= 0.01;
				if release <= 0.0 {
					continue
				}
			}
			self.playkeys.insert(key, playkey);
		}
	}

	pub fn keydown(&mut self, note: u8, velocity: u8) {
		let (sample_note, vbufs) = match self.buffers
			.iter()
			.enumerate()
			.find(|(_, x)| x.0 > note)
		{
			None => &self.buffers.last().unwrap(),
			Some((idx, _)) => if idx == 0 {
				&self.buffers[0]
			} else {
				&self.buffers[idx - 1]
			},
		};
		let buffer = match vbufs.iter().enumerate().find(|(_, x)| x.0 > velocity) {
			None => &vbufs.last().unwrap().1,
			Some((idx, _)) => if idx == 0 {
				&vbufs[0].1
			} else {
				&vbufs[idx - 1].1
			}
		};
		let playkey = Playkey {
			buffer: buffer.clone(),
			sample_offset: 0.0,
			step: 2f32.powf((note as f32 - *sample_note as f32) / 12.0),
			release: None,
		};
		self.playkeys.insert(note, playkey);
	}

	pub fn keyup(&mut self, note: u8) {
		if let Some(mut playkey) = self.playkeys.get_mut(&note) {
			playkey.release = Some(1.0);
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
