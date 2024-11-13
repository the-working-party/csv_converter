use crate::{config::Config, process};

#[derive(Debug, PartialEq)]
pub struct CsvLine {
	config: Config,
	cell: String,
	lines: Vec<Vec<String>>,
	output: String,
}

impl CsvLine {
	pub fn new(config: Config) -> Self {
		Self {
			config,
			cell: String::new(),
			lines: Vec::new(),
			output: String::new(),
		}
	}

	pub fn parse_line(&mut self, line: &str, is_heading: bool) -> String {
		if is_heading {
			return format!("{}\n", self.config.heading);
		}

		self.lines.clear();
		self.cell.clear();
		let mut in_quotes = false;
		let mut chars = line.chars().peekable();
		let mut line = Vec::new();

		while let Some(c) = chars.next() {
			match c {
				'"' => {
					if in_quotes {
						if chars.peek() == Some(&'"') {
							self.cell.push('"');
							chars.next();
						} else {
							in_quotes = false;
						}
					} else {
						in_quotes = true;
					}
				},
				',' if !in_quotes => {
					line.push(std::mem::take(&mut self.cell));
					self.cell.clear();
				},
				_ => self.cell.push(c),
			}
		}
		line.push(std::mem::take(&mut self.cell));
		self.lines = process::run(&[line], &self.config);
		self.export()
	}

	fn quote_csv_cell(cell: &str) -> String {
		let mut needs_quotes = false;
		let mut contains_quote = false;

		for c in cell.chars() {
			match c {
				',' | '\n' => needs_quotes = true,
				'"' => {
					needs_quotes = true;
					contains_quote = true;
				},
				_ => {},
			}
		}

		if needs_quotes {
			let mut escaped_cell = String::with_capacity(cell.len() + 2);
			escaped_cell.push('"');
			if contains_quote {
				for c in cell.chars() {
					if c == '"' {
						escaped_cell.push('"'); // Escape quote
					}
					escaped_cell.push(c);
				}
			} else {
				escaped_cell.push_str(cell);
			}
			escaped_cell.push('"');
			escaped_cell
		} else {
			cell.to_string()
		}
	}

	fn export(&mut self) -> String {
		self.output.clear();
		let mut first_cell;

		for line in &self.lines {
			first_cell = true;
			for cell in line {
				if !first_cell {
					self.output.push(',');
				}
				first_cell = false;
				self.output.push_str(&Self::quote_csv_cell(cell));
			}
			self.output.push('\n');
		}

		self.output.clone()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::config::Item;

	#[test]
	fn new_test() {
		let config = Config {
			heading: String::from("heading"),
			lines: Vec::new(),
		};

		assert_eq!(
			CsvLine::new(config.clone()),
			CsvLine {
				config,
				cell: String::new(),
				lines: Vec::new(),
				output: String::new(),
			}
		);
	}

	#[test]
	fn parse_line_test() {
		let mut csv = CsvLine::new(Config {
			heading: String::from("h1,h2,h3"),
			lines: vec![vec![Item::Cell(0), Item::Cell(1), Item::Cell(2)]],
		});
		assert_eq!(csv.parse_line("1,2,3", false), String::from("1,2,3\n"));

		assert_eq!(
			csv.parse_line(r#"Jane Doe,"123 Main St, Apt 4","Likes to say ""Hello, World!""""#, false),
			String::from("Jane Doe,\"123 Main St, Apt 4\",\"Likes to say \"\"Hello, World!\"\"\"\n")
		);
	}

	#[test]
	fn quote_csv_cell_test() {
		assert_eq!(CsvLine::quote_csv_cell("test"), String::from("test"));
		assert_eq!(CsvLine::quote_csv_cell("test,"), String::from("\"test,\""));
		assert_eq!(CsvLine::quote_csv_cell("test\""), String::from("\"test\"\"\""));
		assert_eq!(CsvLine::quote_csv_cell("test\ntest"), String::from("\"test\ntest\""));
	}

	#[test]
	fn export_test() {
		assert_eq!(
			CsvLine {
				config: Config::default(),
				cell: String::new(),
				lines: Vec::new(),
				output: String::new(),
			}
			.export(),
			String::from(""),
		);

		assert_eq!(
			CsvLine {
				config: Config::default(),
				cell: String::new(),
				lines: vec![
					vec![String::from("A"), String::from("B"), String::from("C")],
					vec![String::from("D"), String::from("E"), String::from("F")],
				],
				output: String::new(),
			}
			.export(),
			String::from("A,B,C\nD,E,F\n"),
		);
	}
}
