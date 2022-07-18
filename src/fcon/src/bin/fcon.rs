use std::collections::HashMap;

fn main() {
	type Key = (String, bool);
	let mut alias_table: HashMap<Key, Vec<String>> = Default::default();
	let (client, _status) = jack::Client::new(
		"midk_fcon",
		jack::ClientOptions::NO_START_SERVER,
	).unwrap();

	let conf_path = std::env::var("XDG_CONFIG_HOME").unwrap();
	let conf_path = format!("{}/midk/fcon.conf", conf_path);
	let conf = std::fs::read_to_string(conf_path).unwrap();
	let mut key: Option<Key> = None;
	let mut value = Vec::new();
	for line in conf.split('\n') {
		if line.trim().is_empty() {
			if let Some(key) = key.take() {
				alias_table.insert(key, std::mem::take(&mut value));
			}
			continue
		}
		if key.is_none() {
			let split: Vec<&str> = line.split_whitespace().collect();
			key = Some((split[0].to_string(), split[1] == "in"));
		} else {
			value.push(line.to_string());
		}
	}

	let mut iter = std::env::args();
	iter.next();
	let mut last_port = iter.next().unwrap();
	while let Some(next_port) = iter.next() {
		let ins = alias_table.get(&(last_port, false)).unwrap();
		let outs = alias_table.get(&(next_port.clone(), true)).unwrap();
		if ins.len() != outs.len() {
			std::process::exit(1);
		}
		for (cin, cout) in ins.iter().zip(outs.iter()) {
			eprintln!("{} -> {}", cin, cout);
			client.connect_ports_by_name(cin, cout).unwrap();
		}
		last_port = next_port;
	}
}