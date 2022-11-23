use std::collections::HashMap;

fn main() {
	type Key = (String, bool);
	let mut alias_table: HashMap<Key, Vec<String>> = Default::default();
	let (client, _status) =
		jack::Client::new("midk_fcon", jack::ClientOptions::NO_START_SERVER)
			.unwrap();

	let args = aarg::parse().unwrap();
	let conf_path = match args.get("--config") {
		Some(vs) => vs[0].clone(),
		None => {
			let conf_path = std::env::var("HPM_ROOT").unwrap();
			format!("{}/asrcpq/midk/fcon.conf", conf_path)
		}
	};
	let disconnect = args.get("--disconnect").is_some();
	let conf = std::fs::read_to_string(conf_path).unwrap();
	let mut key: Option<Key> = None;
	let mut value = Vec::new();
	for line in conf.split('\n') {
		if line.trim().is_empty() {
			if let Some(key) = key.take() {
				alias_table.insert(key, std::mem::take(&mut value));
			}
			continue;
		}
		if key.is_none() {
			let split: Vec<&str> = line.split_whitespace().collect();
			key = Some((split[0].to_string(), split[1] == "in"));
		} else {
			value.push(line.to_string());
		}
	}

	let mut iter = args.get("").unwrap()[1..].iter();
	let mut last_port = iter.next().unwrap();
	for next_port in iter {
		let ins = alias_table
			.get(&(last_port.clone(), false))
			.unwrap_or_else(|| panic!("{} not found", last_port));
		let outs = alias_table
			.get(&(next_port.clone(), true))
			.unwrap_or_else(|| panic!("{} not found", next_port));
		if ins.len() != outs.len() {
			std::process::exit(1);
		}
		for (cin, cout) in ins.iter().zip(outs.iter()) {
			let result = if disconnect {
				client.disconnect_ports_by_name(cin, cout)
			} else {
				client.connect_ports_by_name(cin, cout)
			};
			match result {
				Ok(_) => eprintln!("Ok {} -> {}", cin, cout),
				Err(jack::Error::PortAlreadyConnected(a, b)) => {
					eprintln!("Exist {} -> {}", a, b);
				}
				Err(jack::Error::PortDisconnectionError) => {
					eprintln!("Nonexist {} -> {}", cin, cout);
				}
				Err(e) => panic!("{}", e),
			}
		}
		last_port = next_port;
	}
}
