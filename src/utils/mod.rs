use std::env;

pub fn parse_file(path: &str) -> Vec<String> {
	let mut channels = Vec::new();
	let file = std::fs::read_to_string(path).expect("Failed to read file");
	for line in file.lines() {
		channels.push(line.to_string());
	}
	channels
}

pub fn parse_env_list(key: &str) -> Vec<String> {
	match env::var(key) {
		Ok(val) => val,
		Err(_) => {
			panic!("{} not found in environment", key);
		}
	}
	.split(",")
	.map(|s| s.to_string())
	.collect::<Vec<String>>()
}
