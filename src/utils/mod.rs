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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_parse_file() {
		let data = "test1\ntest2\ntest3";
		std::fs::write("./test.txt", data).expect("Failed to write file");

		let channels = parse_file("./test.txt");
		assert_eq!(channels, vec!["test1", "test2", "test3"]);

		std::fs::remove_file("./test.txt").expect("Failed to remove file");
	}

	#[test]
	fn test_parse_env_list() {
		env::set_var("TEST_ENV", "#test1,test2,test3");
		let channels = parse_env_list("TEST_ENV");
		assert_eq!(channels, vec!["#test1", "test2", "test3"]);
	}
}
