use midly::TrackEventKind as Tek;
use midly::MidiMessage as Mm;

use crate::event::UmeEvent;
use crate::content::UmeContent;

#[derive(Clone)]
pub struct UmeDocument {
	pub events: Vec<UmeEvent>,
}

impl UmeDocument {
	pub fn from_bytes(bytes: &[u8]) -> Self {
		let mut read_head = true;
		let mut dt: u32 = 0;
		let mut mt: u32 = 0;
		let mut len: u32 = 0;
		let mut idx: usize = 0;
		let mut events: Vec<UmeEvent> = Vec::new();
		loop {
			if idx >= bytes.len() { break }
			if read_head {
				dt = u32::from_le_bytes(bytes[idx..idx + 4].try_into().unwrap());
				mt = u32::from_le_bytes(bytes[idx + 4..idx + 8].try_into().unwrap());
				len = u32::from_le_bytes(bytes[idx + 8..idx + 12].try_into().unwrap());
				read_head = false;
				idx += 12;
			} else {
				let event = UmeEvent {
					dt,
					mt,
					content: UmeContent::from_bytes(mt, &bytes[idx..idx + len as usize]),
				};
				idx += len as usize;
				read_head = true;
				events.push(event);
			}
		}
		Self { events }
	}

	pub fn from_smf(bytes: &[u8]) -> Self {
		let mut events = Vec::new();
		let smf = midly::Smf::parse(bytes).unwrap();
		let tpb = match smf.header.timing {
			midly::Timing::Metrical(x) => x.as_int(),
			midly::Timing::Timecode(_, _) => unimplemented!(),
		};
		let tracks = smf.tracks;
		eprintln!("tracks: {}", tracks.len());
		for track in tracks.into_iter() {
			let mut uspb;
			let mut uspt = 1000f32;
			for track_event in track.into_iter() {
				let dt = track_event.delta.as_int();
				let dt = (uspt * dt as f32) as u32;
				match track_event.kind {
					Tek::Meta(midly::MetaMessage::Tempo(t)) => {
						uspb = t.as_int();
						uspt = uspb as f32 / tpb as f32;
					}
					Tek::Midi {
						message,
						..
					} => {
						let ume_content = match message {
							Mm::NoteOn {
								key,
								vel,
							} => {
								let vel = vel.as_int();
								if vel == 0 {
									UmeContent::NoteOff(
										key.as_int(),
									)
								} else {
									UmeContent::NoteOn(
										key.as_int(),
										vel as f32 / 128 as f32
									)
								}
							}
							Mm::NoteOff {
								key,
								vel: _,
							} => {
								UmeContent::NoteOff(key.as_int())
							}
							Mm::Controller {
								controller,
								value,
							} => {
								if controller.as_int() == 64 {
									UmeContent::Sustain(value.as_int() >= 64)
								} else {
									eprintln!("skip mm cc {:?}", controller);
									continue
								}
							}
							m => {
								eprintln!("skip mm {:?}", m);
								continue
							}
						};
						events.push(UmeEvent {
							dt,
							mt: ume_content.mt(),
							content: ume_content,
						});
					}
					e => {
						eprintln!("skip {:?}", e);
					},
				}
			}
		}
		Self { events }
	}

	pub fn to_bytes(&self) -> Vec<u8> {
		let mut result = Vec::new();
		for event in self.events.iter() {
			result.extend(event.dt.to_le_bytes().into_iter());
			result.extend(event.mt.to_le_bytes().into_iter());
			let content_bytes = event.content.to_bytes();
			result.extend((content_bytes.len() as u32).to_le_bytes().into_iter());
			result.extend(content_bytes.into_iter());
		}
		result
	}
}
