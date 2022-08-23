use midk_usm::document::UsmDocument;
use midk_usm::ume::UmeContent;

fn main() {
	let path = {
		let mut iter = std::env::args();
		iter.next();
		iter.next().unwrap()
	};
	let data = std::fs::read(path).unwrap();
	let _doc = UsmDocument::<UmeContent>::from_smf(&data);
}
