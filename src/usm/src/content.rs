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

	pub fn to_bytes(&self) -> Vec<u8> {
		match self {
			Self::NoteOn(note, vel) => {
				let mut result = vec![*note];
				result.extend(vel.to_le_bytes().into_iter());
				result
			},
			Self::NoteOff(note) => vec![*note],
			Self::Sustain(up) => vec![*up as u8],
			_ => unimplemented!(),
		}
	}

	pub fn from_bytes(mt: u32, bytes: &[u8]) -> Self {
		match mt {
			1 => Self::NoteOn(bytes[0], f32::from_le_bytes(bytes[1..5].try_into().unwrap())),
			2 => Self::NoteOff(bytes[0]),
			3 => Self::Sustain(bytes[0] == 1),
			_ => unimplemented!(),
		}
	}
}
