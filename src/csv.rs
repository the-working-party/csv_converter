use std::io::BufRead;

pub struct CsvParser<R: BufRead> {
	reader: R,
	buffer: String,
	temp_line: String,
	in_quotes: bool,
	pub bytes_read: u128,
}

impl<R: BufRead> CsvParser<R> {
	pub fn new(reader: R) -> Self {
		Self {
			reader,
			buffer: String::new(),
			temp_line: String::new(),
			in_quotes: false,
			bytes_read: 0,
		}
	}

	pub fn parse_csv_line(&self) -> Vec<String> {
		let mut cell = String::new();
		let mut in_quotes = false;
		let mut chars = self.buffer.chars().peekable();
		let mut record = Vec::new();

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
					record.push(std::mem::take(&mut cell));
					cell.clear();
				},
				_ => cell.push(c),
			}
		}
		record.push(cell);
		record
	}
}

pub fn export(records: &[Vec<String>], output: &mut String) {
	output.clear();
	for line in records {
		let mut first_cell = true;
		for cell in line {
			if !first_cell {
				output.push(',');
			}
			first_cell = false;
			quote_csv_cell(cell, output);
		}
		output.push('\n');
	}
}

fn quote_csv_cell(cell: &str, output: &mut String) {
	let needs_quotes = cell.contains(&[',', '\n', '"'][..]);
	if needs_quotes {
		output.push('"');
		for c in cell.chars() {
			if c == '"' {
				output.push('"');
			}
			output.push(c);
		}
		output.push('"');
	} else {
		output.push_str(cell);
	}
}

impl<R: BufRead> Iterator for CsvParser<R> {
	type Item = Vec<String>;

	fn next(&mut self) -> Option<Self::Item> {
		self.buffer.clear();
		loop {
			self.temp_line.clear();
			match self.reader.read_line(&mut self.temp_line) {
				Ok(0) => {
					if self.buffer.is_empty() {
						return None;
					} else {
						break;
					}
				},
				Ok(bytes) => {
					self.bytes_read += bytes as u128;
					if !self.buffer.is_empty() {
						self.buffer.push('\n');
					}
					self.buffer.push_str(self.temp_line.trim_end());

					let mut i = 0;
					let bytes = self.temp_line.as_bytes();
					while i < bytes.len() {
						if bytes[i] == b'"' {
							let mut quote_count = 1;
							while i + 1 < bytes.len() && bytes[i + 1] == b'"' {
								quote_count += 1;
								i += 1;
							}
							if quote_count % 2 != 0 {
								self.in_quotes = !self.in_quotes;
							}
						}
						i += 1;
					}

					if !self.in_quotes {
						break;
					}
				},
				Err(error) => {
					eprintln!("Error reading line: {}", error);
					return None;
				},
			}
		}

		Some(self.parse_csv_line())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::cli::exit_with_error;
	use std::{fs::File, io::BufReader};

	#[test]
	fn csvparser_iterator_test() {
		let input_file = match File::open("tests/input.csv") {
			Ok(file) => file,
			Err(_) => {
				exit_with_error(1);
			},
		};
		let reader = BufReader::new(input_file);
		let mut csv_file = CsvParser::new(reader);

		let next = csv_file.next();
		assert_eq!(
			next,
			Some(vec![
				String::from("Name"),
				String::from("Address"),
				String::from("Note"),
				String::from("HTML")
			])
		);

		let next = csv_file.next();
		assert_eq!(
			next,
			Some(vec![
				String::from("Jane Doe"),
				String::from("123 Main St, Apt 4"),
				String::from("Likes to say \"Hello, World!\""),
				String::from("<ul>\n<li>A</li>\n<li>B</li>\n</ul>")
			])
		);

		let next = csv_file.next();
		assert_eq!(
			next,
			Some(vec![
				String::from("John Doe"),
				String::from("42 Willborough St"),
				String::from("He's ok"),
				String::from("<span></span>")
			])
		);

		let next = csv_file.next();
		assert_eq!(next, None);

		let input_file = match File::open("tests/config.csv") {
			Ok(file) => file,
			Err(_) => {
				exit_with_error(1);
			},
		};
		let reader = BufReader::new(input_file);
		let mut csv_file = CsvParser::new(reader);

		csv_file.next();
		let next = csv_file.next();
		assert_eq!(
			next,
			Some(vec![
				String::from("<cell1 UPPER_CASE SPLIT|' '|0>"),
				String::from("NEW"),
				String::from("<cell3 REPLACE|'\"'|'\\''>"),
				String::from("<cell2>")
			])
		);
		let mut output = String::new();
		export(&[next.unwrap()], &mut output);
		assert_eq!(output, String::from("<cell1 UPPER_CASE SPLIT|' '|0>,NEW,\"<cell3 REPLACE|'\"\"'|'\\''>\",<cell2>\n"));
	}

	#[test]
	fn export_test() {
		let input_file = match File::open("tests/input.csv") {
			Ok(file) => file,
			Err(_) => {
				exit_with_error(1);
			},
		};
		let reader = BufReader::new(input_file);
		let mut csv_file = CsvParser::new(reader);
		let mut output = String::new();

		let next = csv_file.next();
		export(&[next.unwrap()], &mut output);
		assert_eq!(output, String::from("Name,Address,Note,HTML\n"));

		let next = csv_file.next();
		export(&[next.unwrap()], &mut output);
		assert_eq!(
			output,
			String::from(
				"Jane Doe,\"123 Main St, Apt 4\",\"Likes to say \"\"Hello, World!\"\"\",\"<ul>\n<li>A</li>\n<li>B</li>\n</ul>\"\n"
			)
		);

		let next = csv_file.next();
		export(&[next.unwrap()], &mut output);
		assert_eq!(output, String::from("John Doe,42 Willborough St,He's ok,<span></span>\n"));
	}

	#[test]
	fn quote_csv_cell_test() {
		let mut output = String::from("");
		quote_csv_cell("test", &mut output);
		assert_eq!(output, String::from("test"));
		output.clear();
		quote_csv_cell("test,", &mut output);
		assert_eq!(output, String::from("\"test,\""));
		output.clear();
		quote_csv_cell("test\"", &mut output);
		assert_eq!(output, String::from("\"test\"\"\""));
		output.clear();
		quote_csv_cell("test\ntest", &mut output);
		assert_eq!(output, String::from("\"test\ntest\""));
	}
}
