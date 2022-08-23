pub fn ascii_print(data: &[u8]) -> String {
	data.iter()
		.map(|x| if x.is_ascii_graphic() {
			*x as char
		} else {
			'?'
		}).collect()
}

#[derive(Clone)]
pub struct UsmEvent<C> {
	pub dt: u32,
	pub mt: u32,
	pub data: C,
}
