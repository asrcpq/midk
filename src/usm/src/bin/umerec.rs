// umerec does not have precise alignment, use it for only recording raw instrument
// do not use it for exporting

use midk_usm::content::UmeContent as Uc;
use midk_usm::event::UmeEvent;
use midk_usm::document::UmeDocument;
use std::sync::{Arc, Mutex};
use std::io::BufRead;

fn main() {
	let path = {
		let mut iter = std::env::args();
		iter.next();
		iter.next().unwrap()
	};
	let (client, _status) = jack::Client::new(
		"midk_umerec",
		jack::ClientOptions::NO_START_SERVER,
	).unwrap();
	let events = Arc::new(Mutex::new(Vec::new()));
	let events2 = events.clone();
	let sr = client.sample_rate();
	let bs = client.buffer_size() as i64;
	let mut last_frame = 0i64;
	let midi_in = client
		.register_port("midi_in", jack::MidiIn::default())
		.unwrap();
	let callback =
		move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
			let mut events_new = Vec::new();
			for event in midi_in.iter(ps) {
				let bytes = event.bytes;
				let uc = match bytes[0] {
					144..=159 => Uc::NoteOn(bytes[1], bytes[2] as f32 / 128.0),
					128..=143 => Uc::NoteOff(bytes[1]),
					176..=191 => if bytes[1] == 64 {
						Uc::Sustain(bytes[2] >= 64)
					} else {
						continue
					}
					_ => continue,
				};
				eprintln!("{:?}", uc);
				let frame = event.time as i64;
				let dt = ((frame - last_frame) * 1_000_000 / sr as i64) as u32;
				events_new.push(UmeEvent {
					dt,
					mt: uc.mt(),
					content: uc,
				});
				last_frame = frame;
			}
			last_frame -= bs;
			events.lock().unwrap().extend(events_new);
			jack::Control::Continue
		};
	let active_client = client
		.activate_async((), jack::ClosureProcessHandler::new(callback))
		.unwrap();
	let stdin = std::io::stdin();
	let _ = stdin.lock().lines().next().unwrap();
	active_client.deactivate().unwrap();
	let doc = UmeDocument {
		events: events2.lock().unwrap().clone()
	};
	let data = doc.to_bytes();
	std::fs::write(path, data).unwrap();
}
