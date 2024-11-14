use std::io::BufRead;

use crate::{cli::exit_with_error, cli::Settings, config::OutputConfig, process};

#[derive(Debug, PartialEq)]
pub struct CsvLine {
	output_config: OutputConfig,
	cell: String,
	lines: Vec<Vec<String>>,
	output: String,
}

impl CsvLine {
	pub fn read_csv_record(
		reader: &mut dyn BufRead,
		line: &mut String,
		temp_line: &mut String,
		bytes_read: &mut u128,
		settings: &Settings,
	) -> bool {
		line.clear();
		let mut in_quotes = false;

		loop {
			temp_line.clear();
			let bytes = match reader.read_line(temp_line) {
				Ok(bytes) => bytes,
				Err(error) => {
					eprintln!("Error: Failed to read from input file '{}': {error}", settings.input);
					exit_with_error(1);
				},
			};

			if bytes == 0 {
				if line.is_empty() {
					return false;
				} else {
					break;
				}
			}

			*bytes_read += bytes as u128;

			if !line.is_empty() {
				line.push('\n');
			}
			line.push_str(temp_line);

			let mut i = 0;
			let bytes = temp_line.as_bytes();
			while i < bytes.len() {
				if bytes[i] == b'"' {
					let mut quote_count = 1;
					while i + 1 < bytes.len() && bytes[i + 1] == b'"' {
						quote_count += 1;
						i += 1;
					}
					if quote_count % 2 != 0 {
						in_quotes = !in_quotes;
					}
				}
				i += 1;
			}

			if !in_quotes {
				break;
			}
		}

		true
	}

	pub fn new(output_config: OutputConfig) -> Self {
		Self {
			output_config,
			cell: String::new(),
			lines: Vec::new(),
			output: String::new(),
		}
	}

	pub fn parse_line(&mut self, line: &str, is_heading: bool) -> String {
		if is_heading {
			return format!("{}\n", self.output_config.heading);
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
		self.lines = process::run(&[line], &self.output_config);
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
	use std::{fs::File, io::BufReader};

	#[test]
	fn read_csv_record_test() {
		let input_file = match File::open("tests/input.csv") {
			Ok(file) => file,
			Err(_) => {
				exit_with_error(1);
			},
		};
		let mut reader = BufReader::new(input_file);
		let mut line = String::new();
		let mut temp_line = String::new();
		let mut bytes_read: u128 = 0;
		let settings = Settings {
			input: String::new(),
			output: String::new(),
			output_config: String::new(),
			version: false,
			help: false,
		};

		let result = CsvLine::read_csv_record(&mut reader, &mut line, &mut temp_line, &mut bytes_read, &settings);
		assert_eq!(result, true);
		assert_eq!(line, String::from("Name,Address,Note,HTML\n"));

		let result = CsvLine::read_csv_record(&mut reader, &mut line, &mut temp_line, &mut bytes_read, &settings);
		assert_eq!(result, true);
		assert_eq!(line, String::from("Jane Doe,\"123 Main St, Apt 4\",\"Likes to say \"\"Hello, World!\"\"\",\"<ul>\n\n<li>A</li>\n\n<li>B</li>\n\n</ul>\"\n"));

		let result = CsvLine::read_csv_record(&mut reader, &mut line, &mut temp_line, &mut bytes_read, &settings);
		assert_eq!(result, true);
		assert_eq!(line, String::from("John Doe,42 Willborough St,He's ok,\"<span></span>\"\n"));

		let result = CsvLine::read_csv_record(&mut reader, &mut line, &mut temp_line, &mut bytes_read, &settings);
		assert_eq!(result, false);
		assert_eq!(line, String::new());
	}

	#[test]
	fn new_test() {
		let output_config = OutputConfig {
			heading: String::from("heading"),
			lines: Vec::new(),
		};

		assert_eq!(
			CsvLine::new(output_config.clone()),
			CsvLine {
				output_config,
				cell: String::new(),
				lines: Vec::new(),
				output: String::new(),
			}
		);
	}

	#[test]
	fn parse_line_test() {
		let mut csv = CsvLine::new(OutputConfig {
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
				output_config: OutputConfig::default(),
				cell: String::new(),
				lines: Vec::new(),
				output: String::new(),
			}
			.export(),
			String::from(""),
		);

		assert_eq!(
			CsvLine {
				output_config: OutputConfig::default(),
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
