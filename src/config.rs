use std::borrow::Cow;

use crate::cli::exit_with_error;

#[derive(Debug, PartialEq, Clone)]
pub enum Condition {
	_Equals(String),
	_NotEquals(String),
	_GreaterThan(i64),
	_LessThan(i64),
	_Modulo(i64, i64),
	_StartesWith(String),
	_EndsWith(String),
	_Contains(String),
	IsEmpty,
	_IsNotEmpty,
	_IsNumeric,
	// TODO: date functions
}

impl Condition {
	pub fn parse(_condition: String) -> Self {
		Self::IsEmpty
	}
}

#[derive(Debug, PartialEq, Clone)]
pub enum Filter {
	UpperCase,
	LowerCase,
	Split(String, usize),
	SubString(u64, Option<u64>),
	Replace(String, String),
	Append(String),
	Prepend(String),
	Length,
	Trim,
	TrimStart,
	TrimEnd,
}

impl Filter {
	pub fn parse(filter_str: &str) -> Vec<Self> {
		let mut in_quotes = false;
		let mut escaped = false;
		let mut filters_str = Vec::new();
		let mut temp_filter = String::new();
		let mut filters = Vec::new();

		for c in filter_str.trim().chars() {
			match c {
				'\'' => {
					if !escaped {
						in_quotes = true;
					} else {
						escaped = false;
					}
				},
				'\\' => {
					escaped = !escaped;
				},
				' ' => {
					if !in_quotes {
						filters_str.push(temp_filter.clone());
						temp_filter.clear();
						escaped = false;
					} else {
						temp_filter.push(c);
					}
				},
				_ => {
					escaped = false;
					temp_filter.push(c);
				},
			}
		}
		filters_str.push(temp_filter.clone());

		for filter in filters_str {
			match filter.as_str() {
				"UPPER_CASE" => filters.push(Filter::UpperCase),
				"LOWER_CASE" => filters.push(Filter::LowerCase),
				"LENGTH" => filters.push(Filter::Length),
				"TRIM" => filters.push(Filter::Trim),
				"TRIM_START" => filters.push(Filter::TrimStart),
				"TRIM_END" => filters.push(Filter::TrimEnd),
				f if f.starts_with("REPLACE") => {
					let bits = f.split("|").collect::<Vec<&str>>();
					if bits.len() != 3 {
						eprintln!(
							r#"OutputConfig error: Invalid REPLACE filter "{filter}"\n\
							Usage: REPLACE|[string]|[string]\n\
							Example:\n\
							cell1 = "My csv is great"\n\
							<cell1 REPLACE|'great'|'awesome'>\n\
							cell1 = "My csv is awesome""#
						);
						exit_with_error(1);
					}
					filters.push(Filter::Replace(bits[1].to_string(), bits[2].to_string()));
				},
				f if f.starts_with("APPEND") => {
					let bits = f.split("|").collect::<Vec<&str>>();
					if bits.len() != 2 {
						eprintln!(
							r#"OutputConfig error: Invalid APPEND filter "{filter}"\n\
							Usage: REPLACE|[string]\n\
							Example:\n\
							cell1 = "dark"\n\
							<cell1 APPEND|'-brown'>\n\
							cell1 = "dark-brown""#
						);
						exit_with_error(1);
					}
					filters.push(Filter::Append(bits[1].to_string()));
				},
				f if f.starts_with("PREPEND") => {
					let bits = f.split("|").collect::<Vec<&str>>();
					if bits.len() != 2 {
						eprintln!(
							r#"OutputConfig error: Invalid PREPEND filter "{filter}"\n\
							Usage: PREPEND|[string]\n\
							Example:\n\
							cell1 = "Bond"\n\
							<cell1 PREPEND|'James '>\n\
							cell1 = "James Bond""#
						);
						exit_with_error(1);
					}
					filters.push(Filter::Prepend(bits[1].to_string()));
				},
				f if f.starts_with("SPLIT") => {
					let bits = f.split("|").collect::<Vec<&str>>();
					if bits.len() != 3 {
						eprintln!(
							r#"OutputConfig error: Invalid SPLIT filter "{filter}"\n\
							Usage: SPLIT|[string]|[number]\n\
							Example:\n\
							cell1 = "one,two,three,four"\n\
							<cell1 SPLIT|','|3>\n\
							cell1 = "three""#
						);
						exit_with_error(1);
					}
					let index = match bits[2].parse::<usize>() {
						Ok(n) => {
							if n > 0 {
								n
							} else {
								eprintln!(r#"OutputConfig error: The SPLIT index must be positive, was "{}""#, bits[2]);
								exit_with_error(1);
							}
						},
						Err(_) => {
							eprintln!(r#"OutputConfig error: Invalid SPLIT index "{}""#, bits[2]);
							exit_with_error(1);
						},
					};
					filters.push(Filter::Split(bits[1].to_string(), index));
				},
				f if f.starts_with("SUB_STRING") => {
					let bits = f.split("|").collect::<Vec<&str>>();
					if bits.len() != 2 && bits.len() != 3 {
						eprintln!(
							r#"OutputConfig error: Invalid SUB_STRING filter "{filter}"\n\
							Usage: SUB_STRING|[number]|[number optional]\n\
							Example:\n\
							cell1 = "The Working Party"\n\
							<cell1 SPLIT|4>\n\
							cell1 = "Working Party"\n\n\

							cell1 = "The Working Party"\n\
							<cell1 SPLIT|4|7>\n\
							cell1 = "Working""#
						);
						exit_with_error(1);
					}
					let start = match bits[1].parse::<u64>() {
						Ok(n) => n,
						Err(_) => {
							eprintln!(r#"OutputConfig error: Invalid SUB_STRING start "{}""#, bits[1]);
							exit_with_error(1);
						},
					};
					let end = if bits.len() == 3 {
						match bits[2].parse::<u64>() {
							Ok(n) => Some(n),
							Err(_) => {
								eprintln!(r#"OutputConfig error: Invalid SUB_STRING start "{}""#, bits[2]);
								exit_with_error(1);
							},
						}
					} else {
						None
					};
					filters.push(Filter::SubString(start, end));
				},
				_ => {
					eprintln!(r#"OutputConfig error: Filter not recognized "{filter}" and will be ignored."#);
				},
			}
		}

		filters
	}

	pub fn run<'a>(&self, input: Cow<'a, str>) -> Cow<'a, str> {
		match self {
			Self::UpperCase => Cow::Owned(input.to_uppercase()),
			Self::LowerCase => Cow::Owned(input.to_lowercase()),
			Self::Split(_needle, _index) => {
				unimplemented!()
			},
			Self::SubString(_start, _length) => {
				unimplemented!()
			},
			Self::Replace(_search, _replacement) => {
				unimplemented!()
			},
			Self::Append(suffix) => {
				let mut s = input.into_owned();
				s.push_str(suffix);
				Cow::Owned(s)
			},
			Self::Prepend(prefix) => {
				let mut s = prefix.clone();
				s.push_str(&input);
				Cow::Owned(s)
			},
			Self::Length => {
				unimplemented!()
			},
			Self::Trim => Cow::Owned(input.trim().to_string()),
			Self::TrimStart => {
				unimplemented!()
			},
			Self::TrimEnd => {
				unimplemented!()
			},
		}
	}
}

#[derive(Debug, PartialEq, Clone)]
pub enum Item {
	Value(String),
	If(Condition, Option<Box<Item>>),
	Cell(usize, Option<Vec<Filter>>),
}

impl Item {
	pub fn parse(input: String) -> Self {
		if input.starts_with("<cell") && input.ends_with('>') {
			let cell_str = &input[5..input.len() - 1];
			let mut filter = None;
			let num_str = match cell_str.find(' ') {
				Some(index) => {
					filter = Some(Filter::parse(&cell_str[index + 1..]));
					&cell_str[..index]
				},
				None => cell_str,
			};

			match num_str.parse::<usize>() {
				Ok(n) => {
					if n > 0 {
						Item::Cell(n - 1, filter)
					} else {
						eprintln!("OutputConfig error: Cell number must be positive for item '{input}'");
						exit_with_error(1);
					}
				},
				Err(_) => {
					eprintln!("OutputConfig error: Invalid cell number '{input}'");
					exit_with_error(1);
				},
			}
		} else if input.starts_with("=IF ") {
			let trimmed = input.trim_start_matches("=IF ").trim();
			let parts = trimmed.splitn(2, "ELSE").map(str::trim).collect::<Vec<&str>>();
			if parts.is_empty() || parts[0].is_empty() {
				eprintln!("OutputConfig error: Invalid if condition '{input}'");
				exit_with_error(1);
			}
			let condition = parts[0].to_string();
			let else_condition = parts.get(1).map(|&else_condition| Box::new(Item::parse(else_condition.to_string())));

			Item::If(Condition::parse(condition), else_condition)
		} else {
			Item::Value(input.to_string())
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
		// TODO: fix parsing of config so new lines and commas in filters work

		if let Some(header_line) = config_lines.next() {
			heading = header_line.to_string();
		} else {
			eprintln!("OutputConfig error: No content found");
			exit_with_error(1);
		}

		for line in config_lines {
			let mut cells = Vec::new();
			for cell in line.split(",") {
				cells.push(Item::parse(cell.to_string()));
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
			OutputConfig::new("heading1,heading2,heading3\n<cell1>,<cell2>,<cell3>\n"),
			OutputConfig {
				heading: String::from("heading1,heading2,heading3"),
				lines: vec![vec![Item::Cell(0, None), Item::Cell(1, None), Item::Cell(2, None),]],
			}
		);

		assert_eq!(
			OutputConfig::new("h1,h2,h3,h4\n<cell1>,,hardcoded,=IF my condition\n"),
			OutputConfig {
				heading: String::from("h1,h2,h3,h4"),
				lines: vec![vec![
					Item::Cell(0, None),
					Item::Value(String::from("")),
					Item::Value(String::from("hardcoded")),
					Item::If(Condition::IsEmpty, None),
				]],
			}
		);
	}

	#[test]
	fn filter_test() {
		assert_eq!(
			OutputConfig::new("H1,H2,H3\n<cell1 LENGTH>,<cell2 UPPER_CASE LOWER_CASE REPLACE|' '|'-'>,<cell3>\n"),
			OutputConfig {
				heading: String::from("H1,H2,H3"),
				lines: vec![vec![
					Item::Cell(0, Some(vec![Filter::Length])),
					Item::Cell(
						1,
						Some(vec![
							Filter::UpperCase,
							Filter::LowerCase,
							Filter::Replace(String::from(" "), String::from("-"))
						])
					),
					Item::Cell(2, None),
				]],
			}
		);
	}

	#[test]
	fn upper_case_test() {
		assert_eq!(
			OutputConfig::new("H1\n<cell1 UPPER_CASE>\n"),
			OutputConfig {
				heading: String::from("H1"),
				lines: vec![vec![Item::Cell(0, Some(vec![Filter::UpperCase,])),]],
			}
		);

		assert_eq!(Filter::UpperCase.run(Cow::Borrowed("test")), Cow::Borrowed("TEST"));
		assert_eq!(Filter::UpperCase.run(Cow::Borrowed("TEST")), Cow::Borrowed("TEST"));
		assert_eq!(Filter::UpperCase.run(Cow::Borrowed("TeSt 😬")), Cow::Borrowed("TEST 😬"));
	}

	#[test]
	fn lower_case_test() {
		assert_eq!(
			OutputConfig::new("H1\n<cell1 LOWER_CASE>\n"),
			OutputConfig {
				heading: String::from("H1"),
				lines: vec![vec![Item::Cell(0, Some(vec![Filter::LowerCase,])),]],
			}
		);

		assert_eq!(Filter::LowerCase.run(Cow::Borrowed("test")), Cow::Borrowed("test"));
		assert_eq!(Filter::LowerCase.run(Cow::Borrowed("TEST")), Cow::Borrowed("test"));
		assert_eq!(Filter::LowerCase.run(Cow::Borrowed("TeSt 😬")), Cow::Borrowed("test 😬"));
	}

	#[test]
	fn length_test() {
		assert_eq!(
			OutputConfig::new("H1\n<cell1 LENGTH>\n"),
			OutputConfig {
				heading: String::from("H1"),
				lines: vec![vec![Item::Cell(0, Some(vec![Filter::Length,])),]],
			}
		);
	}

	#[test]
	fn trim_test() {
		assert_eq!(
			OutputConfig::new("H1\n<cell1 TRIM>\n"),
			OutputConfig {
				heading: String::from("H1"),
				lines: vec![vec![Item::Cell(0, Some(vec![Filter::Trim,])),]],
			}
		);
	}

	#[test]
	fn trim_start_test() {
		assert_eq!(
			OutputConfig::new("H1\n<cell1 TRIM_START>\n"),
			OutputConfig {
				heading: String::from("H1"),
				lines: vec![vec![Item::Cell(0, Some(vec![Filter::TrimStart,])),]],
			}
		);
	}

	#[test]
	fn trim_end_test() {
		assert_eq!(
			OutputConfig::new("H1\n<cell1 TRIM_END>\n"),
			OutputConfig {
				heading: String::from("H1"),
				lines: vec![vec![Item::Cell(0, Some(vec![Filter::TrimEnd,])),]],
			}
		);
	}

	#[test]
	fn replace_test() {
		assert_eq!(
			OutputConfig::new("H1\n<cell1 REPLACE|'-'|' '>\n"),
			OutputConfig {
				heading: String::from("H1"),
				lines: vec![vec![Item::Cell(
					0,
					Some(vec![Filter::Replace(String::from("-"), String::from(" ")),])
				),]],
			}
		);

		assert_eq!(
			OutputConfig::new("H1\n<cell1 REPLACE|'...'|'##'>\n"),
			OutputConfig {
				heading: String::from("H1"),
				lines: vec![vec![Item::Cell(
					0,
					Some(vec![Filter::Replace(String::from("..."), String::from("##")),])
				),]],
			}
		);
	}

	#[test]
	fn split_test() {
		assert_eq!(
			OutputConfig::new("H1\n<cell1 SPLIT|'-'|6>\n"),
			OutputConfig {
				heading: String::from("H1"),
				lines: vec![vec![Item::Cell(0, Some(vec![Filter::Split(String::from("-"), 6),])),]],
			}
		);

		assert_eq!(
			OutputConfig::new("H1\n<cell1 SPLIT|'###'|666>\n"),
			OutputConfig {
				heading: String::from("H1"),
				lines: vec![vec![Item::Cell(
					0,
					Some(vec![Filter::Split(String::from("###"), 666),])
				),]],
			}
		);
	}

	#[test]
	fn append_test() {
		assert_eq!(
			OutputConfig::new("H1\n<cell1 APPEND|'end'>\n"),
			OutputConfig {
				heading: String::from("H1"),
				lines: vec![vec![Item::Cell(0, Some(vec![Filter::Append(String::from("end")),])),]],
			}
		);

		assert_eq!(
			OutputConfig::new("H1\n<cell1 APPEND|'###'>\n"),
			OutputConfig {
				heading: String::from("H1"),
				lines: vec![vec![Item::Cell(0, Some(vec![Filter::Append(String::from("###")),])),]],
			}
		);

		assert_eq!(Filter::Append(String::from("-end")).run(Cow::Borrowed("middle")), Cow::Borrowed("middle-end"));
		assert_eq!(Filter::Append(String::from("- 😬")).run(Cow::Borrowed("middle")), Cow::Borrowed("middle- 😬"));
	}

	#[test]
	fn prepend_test() {
		assert_eq!(
			OutputConfig::new("H1\n<cell1 PREPEND|'front'>\n"),
			OutputConfig {
				heading: String::from("H1"),
				lines: vec![vec![Item::Cell(0, Some(vec![Filter::Prepend(String::from("front")),])),]],
			}
		);

		assert_eq!(
			OutputConfig::new("H1\n<cell1 PREPEND|'###'>\n"),
			OutputConfig {
				heading: String::from("H1"),
				lines: vec![vec![Item::Cell(0, Some(vec![Filter::Prepend(String::from("###")),])),]],
			}
		);

		assert_eq!(Filter::Prepend(String::from("start-")).run(Cow::Borrowed("middle")), Cow::Borrowed("start-middle"));
		assert_eq!(Filter::Prepend(String::from("😬 -")).run(Cow::Borrowed("middle")), Cow::Borrowed("😬 -middle"));
	}

	#[test]
	fn sub_string_test() {
		assert_eq!(
			OutputConfig::new("H1\n<cell1 SUB_STRING|5>\n"),
			OutputConfig {
				heading: String::from("H1"),
				lines: vec![vec![Item::Cell(0, Some(vec![Filter::SubString(5, None)])),]],
			}
		);

		assert_eq!(
			OutputConfig::new("H1\n<cell1 SUB_STRING|999|666>\n"),
			OutputConfig {
				heading: String::from("H1"),
				lines: vec![vec![Item::Cell(0, Some(vec![Filter::SubString(999, Some(666))])),]],
			}
		);
	}
}
