use midk_polysplit::exsquare::ExsquareGenerator;
use midk_polysplit::poly_host::PolyHost;

fn main() {
	let ph = PolyHost::new(Box::new(ExsquareGenerator::default()));
	ph.run();
}
