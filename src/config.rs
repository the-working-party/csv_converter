use std::fmt;

use crate::cli::exit_with_error;

#[derive(Debug, PartialEq, Clone)]
pub enum Item {
	Value(String),
	If(String),
	Cell(usize),
}

impl fmt::Display for Item {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Item::Value(n) => write!(f, "{n}"),
			Item::Cell(n) => write!(f, "<cell{}>", n + 1),
			Item::If(cond) => write!(f, "=IF({})", cond),
		}
	}
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct OutputConfig {
	pub heading: String,
	pub lines: Vec<Vec<Item>>,
}

impl OutputConfig {
	pub fn new(config: &str) -> Self {
		let mut config_lines = config.lines();
		let heading;
		let mut lines = Vec::new();

		if let Some(header_line) = config_lines.next() {
			heading = header_line.to_string();
		} else {
			eprintln!("OutputConfig error: No content found");
			exit_with_error(1);
		}

		for line in config_lines {
			let mut cells = Vec::new();
			for cell in line.split(",") {
				if cell.starts_with("<cell") && cell.ends_with('>') {
					let num_str = &cell[5..cell.len() - 1];
					match num_str.parse::<usize>() {
						Ok(n) if n > 0 => cells.push(Item::Cell(n - 1)),
						Ok(_) => {
							eprintln!("OutputConfig error: Cell number must be positive for item '{cell}'");
							exit_with_error(1);
						},
						Err(_) => {
							eprintln!("OutputConfig error: Invalid cell number '{cell}'");
							exit_with_error(1);
						},
					};
				} else if cell.starts_with("=IF") {
					if let Some(start) = cell.find('(') {
						if let Some(end) = cell.rfind(')') {
							if end > start {
								cells.push(Item::If(cell[start + 1..end].trim().to_string()));
							}
						}
					}
				} else {
					cells.push(Item::Value(cell.to_string()));
				}
			}
			lines.push(cells);
		}

		Self { heading, lines }
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn new_test() {
		assert_eq!(
			OutputConfig::new("headine1,heading2,heading3\n<cell1>,<cell2>,<cell3>\n"),
			OutputConfig {
				heading: String::from("headine1,heading2,heading3"),
				lines: vec![vec![Item::Cell(0), Item::Cell(1), Item::Cell(2),]],
			}
		);

		assert_eq!(
			OutputConfig::new("h1,h2,h3,h4\n<cell1>,,hardcoded,=IF(my condition)\n"),
			OutputConfig {
				heading: String::from("h1,h2,h3,h4"),
				lines: vec![vec![
					Item::Cell(0),
					Item::Value(String::from("")),
					Item::Value(String::from("hardcoded")),
					Item::If(String::from("my condition")),
				]],
			}
		);
	}
}
