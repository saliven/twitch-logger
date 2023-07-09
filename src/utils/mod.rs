pub fn load_channels() -> Vec<String> {
	let mut channels = Vec::new();
	let file = std::fs::read_to_string("channels.txt").expect("Failed to read channels.txt");
	for line in file.lines() {
		channels.push(line.to_string());
	}
	channels
}
