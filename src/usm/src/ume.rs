use midly::TrackEventKind as Tek;
use midly::MidiMessage as Mm;

use crate::document::UsmDocument;
use crate::event::UsmEvent;

// it is acutally a union
#[derive(Clone, Debug)]
pub enum UmeContent {
	NoteOn(u8, f32),
	NoteOff(u8),
	Sustain(bool),
	Beat(bool),
	Message(String),
}

impl UmeContent {
	pub fn mt(&self) -> u32 {
		match self {
			Self::NoteOn(_, _) => 1,
			Self::NoteOff(_) => 2,
			Self::Sustain(_) => 3,
			Self::Beat(_) => 4,
			Self::Message(_) => 5,
		}
	}
}

impl UsmDocument<UmeContent> {
	pub fn from_smf(bytes: &[u8]) -> Self {
		let mut events = Vec::new();
		let smf = midly::Smf::parse(bytes).unwrap();
		let tpb = match smf.header.timing {
			midly::Timing::Metrical(x) => x.as_int(),
			midly::Timing::Timecode(_, _) => unimplemented!(),
		};
		let mut uspb;
		let mut uspt = 1000f32;
		let track = smf.tracks.into_iter().next().unwrap();
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
							UmeContent::NoteOn(
								key.as_int(),
								vel.as_int() as f32 / 128 as f32
							)
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
					events.push(UsmEvent {
						dt,
						mt: ume_content.mt(),
						data: ume_content,
					});
				}
				e => {
					eprintln!("skip {:?}", e);
				},
			}
		}
		Self { events }
	}
}
