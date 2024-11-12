#[derive(Debug, PartialEq)]
pub struct CsvLine {
	line: Vec<String>,
}

impl CsvLine {
	pub fn new(line: &str) -> Self {
		Self {
			line: Self::parse_line(line),
		}
	}

	pub fn parse_line(line: &str) -> Vec<String> {
		let mut cells = Vec::new();
		let mut in_quotes = false;
		let mut cell = String::new();
		let mut chars = line.chars().peekable();

		while let Some(c) = chars.next() {
			match c {
				'"' => {
					if in_quotes {
						if chars.peek() == Some(&'"') {
							cell.push('"');
							chars.next();
						} else {
							in_quotes = false;
						}
					} else {
						in_quotes = true;
					}
				},
				',' if !in_quotes => {
					cells.push(std::mem::take(&mut cell));
					cell.clear();
				},
				_ => cell.push(c),
			}
		}
		cells.push(cell);
		cells
	}

	pub fn process(mut self) -> Self {
		for _cell in &mut self.line {
			// TODO: do the processing here
		}
		self
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
			let mut escaped_cell = String::with_capacity(cell.len() + 2); // Preallocate space
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

	pub fn export(&self) -> String {
		let mut result = String::new();
		let mut first = true;

		for cell in &self.line {
			if !first {
				result.push(',');
			}
			first = false;
			result.push_str(&Self::quote_csv_cell(cell));
		}
		result.push('\n');
		result
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn new_test() {
		assert_eq!(
			CsvLine::new("1,2,3"),
			CsvLine {
				line: vec![String::from("1"), String::from("2"), String::from("3"),]
			}
		);
		assert_eq!(
			CsvLine::new(r#"Jane Doe,"123 Main St, Apt 4","Likes to say ""Hello, World!""""#),
			CsvLine {
				line: vec![
					String::from("Jane Doe"),
					String::from("123 Main St, Apt 4"),
					String::from("Likes to say \"Hello, World!\"")
				]
			}
		);
	}

	#[test]
	fn parse_line_test() {
		assert_eq!(CsvLine::parse_line("1,2,3"), vec![String::from("1"), String::from("2"), String::from("3")]);
		// assert_eq!(CsvLine::parse_line(r#"test,"#), vec![String::from("1"), String::from("2"), String::from("3")]);
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
		assert_eq!(CsvLine::new("1,2,3").export(), String::from("1,2,3\n"));
		assert_eq!(CsvLine::new(r#"test,"te""st""#).export(), String::from("test,\"te\"\"st\"\n"));
	}
}
