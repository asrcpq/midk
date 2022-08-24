use jack::RawMidi as Rm;
use std::io::BufRead;
use std::sync::mpsc::channel;

use midk_usm::document::UmeDocument;
use midk_usm::content::UmeContent as Uc;
use midk_usm::align::Align;

fn main() {
	let path = {
		let mut iter = std::env::args();
		iter.next();
		iter.next().unwrap()
	};
	let is_midi = match std::path::Path::new(&path)
		.extension()
		.unwrap()
		.to_str()
		.unwrap()
	{
		"mid" => true,
		"midi" => true,
		"ume" => false,
		_ => unimplemented!(),
	};
	let data = std::fs::read(path).unwrap();
	let doc = if is_midi {
		UmeDocument::from_smf(&data)
	} else {
		UmeDocument::from_bytes(&data)
	};
	eprintln!("{} events", doc.events.len());
	let (tx, rx) = channel();
	let mut iter = doc.events.into_iter().enumerate().peekable();
	let (client, _status) = jack::Client::new(
		"midk_umecat",
		jack::ClientOptions::NO_START_SERVER,
	).unwrap();
	let sr = client.sample_rate();
	let bs = client.buffer_size();
	let mut align = Align::new(sr as u32);
	let mut midi_out = client
		.register_port("midi_out", jack::MidiOut::default())
		.unwrap();
	let stdin = std::io::stdin();
	let _ = stdin.lock().lines().next().unwrap();
	let mut cd = 0;
	let callback =
		move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
			let mut writer = midi_out.writer(ps);
			while let Some((idx, event)) = iter.peek() {
				let f = align.get_frame(event.dt);
				if f >= bs {
					let new_cd = f / sr as u32;
					if new_cd != cd {
						eprint!("[2KWait: {}s\r", new_cd + 1);
						cd = new_cd;
					}
					break
				}
				eprintln!("proc event {} {:?}", idx, event);
				match event.content {
					Uc::NoteOn(x, vel) => {
						writer.write(&Rm {
							time: f,
							bytes: &[144, x, (vel * 128.0) as u8],
						}).unwrap();
					}
					Uc::NoteOff(x) => {
						writer.write(&Rm {
							time: f,
							bytes: &[128, x, 0],
						}).unwrap();
					}
					Uc::Sustain(x) => {
						writer.write(&Rm {
							time: f,
							bytes: &[176, 64, x as u8 * 127]
						}).unwrap();
					}
					_ => {},
				}
				align.go_timer(event.dt);
				iter.next();
			}
			if iter.peek().is_none() {
				tx.send(()).unwrap();
				return jack::Control::Quit
			}
			align.go_frame(bs);
			jack::Control::Continue
		};

	let active_client = client
		.activate_async((), jack::ClosureProcessHandler::new(callback))
		.unwrap();
	let _ = rx.recv();
	active_client.deactivate().unwrap();
}
