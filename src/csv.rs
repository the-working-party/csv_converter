#[derive(Debug, PartialEq)]
pub struct CsvLine {
	config: String,
	cell: String,
	line: Vec<String>,
	output: String,
}

impl CsvLine {
	pub fn new(config: String) -> Self {
		Self {
			config,
			cell: String::new(),
			line: Vec::new(),
			output: String::new(),
		}
	}

	pub fn parse_line(&mut self, line: &str, _is_heading: bool) -> String {
		self.line.clear();
		self.cell.clear();
		let mut in_quotes = false;
		let mut chars = line.chars().peekable();

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
					self.line.push(std::mem::take(&mut self.cell));
					self.cell.clear();
				},
				_ => self.cell.push(c),
			}
		}
		self.line.push(std::mem::take(&mut self.cell));
		self.process();
		self.export()
	}

	fn process(&mut self) {
		// for _cell in self.line {
		// 	// TODO: do the processing here
		// }
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
		let mut first = true;

		for cell in &self.line {
			if !first {
				self.output.push(',');
			}
			first = false;
			self.output.push_str(&Self::quote_csv_cell(cell));
		}
		self.output.push('\n');

		self.output.clone()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn new_test() {
		assert_eq!(
			CsvLine::new(String::from("config")),
			CsvLine {
				config: String::from("config"),
				cell: String::new(),
				line: Vec::new(),
				output: String::new(),
			}
		);
	}

	#[test]
	fn parse_line_test() {
		let mut csv = CsvLine::new(String::from("config"));
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
}
