use crate::event::UsmEvent;

#[derive(Clone)]
pub struct UsmDocument<T: Clone> {
	pub events: Vec<UsmEvent<T>>,
}

impl UsmDocument<Vec<u8>> {
	pub fn from_bytes(bytes: &[u8]) -> Self {
		let mut read_head = true;
		let mut dt: u32 = 0;
		let mut mt: u32 = 0;
		let mut len: u32 = 0;
		let mut idx: usize = 0;
		let mut events: Vec<UsmEvent<Vec<u8>>> = Vec::new();
		loop {
			if idx >= bytes.len() { break }
			if read_head {
				dt = u32::from_le_bytes(bytes[idx..idx + 4].try_into().unwrap());
				mt = u32::from_le_bytes(bytes[idx + 4..idx + 8].try_into().unwrap());
				len = u32::from_le_bytes(bytes[idx + 8..idx + 12].try_into().unwrap());
				read_head = false;
				idx += 12;
			} else {
				let event = UsmEvent {
					dt,
					mt,
					data: bytes[idx..idx + len as usize].to_vec()
				};
				idx += len as usize;
				read_head = true;
				events.push(event);
			}
		}
		Self { events }
	}

	pub fn to_bytes(&self) -> Vec<u8> {
		let mut result = Vec::new();
		for event in self.events.iter() {
			result.extend(event.dt.to_le_bytes().into_iter());
			result.extend(event.mt.to_le_bytes().into_iter());
			result.extend((event.data.len() as u32).to_le_bytes().into_iter());
			result.extend(event.data.clone().into_iter());
		}
		result
	}
}
