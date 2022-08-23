use midk_usm::document::UmeDocument;

fn main() {
	let path = {
		let mut iter = std::env::args();
		iter.next();
		iter.next().unwrap()
	};
	let data = std::fs::read(path).unwrap();
	let doc = UmeDocument::from_smf(&data);
	eprintln!("events: {}", data.len());
	let data = doc.to_bytes();
	eprintln!("bytes: {}", data.len());
	std::fs::write("/tmp/tmp.ume", &data).unwrap();
}
