pub fn parse_file(path: &str) -> Vec<String> {
	let mut channels = Vec::new();
	let file = std::fs::read_to_string(path).expect("Failed to read channels.txt");
	for line in file.lines() {
		channels.push(line.to_string());
	}
	channels
}
