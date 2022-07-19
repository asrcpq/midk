fn main() {
	let (client, _status) =
		jack::Client::new("midk_lsp", jack::ClientOptions::NO_START_SERVER)
			.unwrap();

	for port in client.ports(None, None, jack::PortFlags::empty()) {
		println!("{}", port);
	}
}
