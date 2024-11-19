use std::{borrow::Cow, io::BufRead};

use crate::{
	cli::exit_with_error,
	csv::{self, CsvParser},
};

#[derive(Debug, PartialEq, Clone)]
pub enum Condition {
	_Equals(String, Box<Item>),
	_NotEquals(String, Box<Item>),
	_GreaterThan(i64, Box<Item>),
	_LessThan(i64, Box<Item>),
	_Modulo(i64, i64, Box<Item>),
	_StartesWith(String, Box<Item>),
	_EndsWith(String, Box<Item>),
	_Contains(String, Box<Item>),
	IsEmpty(Box<Item>),
	_IsNotEmpty(Box<Item>),
	_IsNumeric(Box<Item>),
	// TODO: date functions
}

impl Condition {
	pub fn parse(_condition_str: &str) -> (Self, Option<Box<Item>>) {
		// "<celle1 REPLACE|' '|'-' SUB_STRING|10|5> [condition] <cell2> ELSE <cell3>"
		// == 'this item'
		// != 'this item'
		// > 42
		// < 42
		// % 2 = 0
		// STARTS_WITH|'beginning'
		// IS_EMPTY

		// let condition;
		// let else_item;
		// let in_cell;
		// let in_quote;
		// let escaped;

		// for c in condition_str.trim().chars() {
		// 	let if_item;
		// }

		(Self::IsEmpty(Box::new(Item::Value(String::from("")))), None)
	}
}

#[derive(Debug, PartialEq, Clone)]
pub enum Filter {
	UpperCase,
	LowerCase,
	Length,
	Trim,
	TrimStart,
	TrimEnd,
	Replace(String, String),
	Append(String),
	Prepend(String),
	Split(String, usize),
	SubString(usize, Option<usize>),
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
						in_quotes = !in_quotes;
					} else {
						temp_filter.push(c);
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
							"OutputConfig error: Invalid REPLACE filter \"{filter}\"\n\
							Usage: REPLACE|[string]|[string]\n\
							Example:\n\
							cell1 = \"My csv is great\"\n\
							<cell1 REPLACE|'great'|'awesome'>\n\
							cell1 = \"My csv is awesome\""
						);
						exit_with_error(1);
					}
					filters.push(Filter::Replace(bits[1].to_string(), bits[2].to_string()));
				},
				f if f.starts_with("APPEND") => {
					let bits = f.split("|").collect::<Vec<&str>>();
					if bits.len() != 2 {
						eprintln!(
							"OutputConfig error: Invalid APPEND filter \"{filter}\"\n\
							Usage: REPLACE|[string]\n\
							Example:\n\
							cell1 = \"dark\"\n\
							<cell1 APPEND|'-brown'>\n\
							cell1 = \"dark-brown\""
						);
						exit_with_error(1);
					}
					filters.push(Filter::Append(bits[1].to_string()));
				},
				f if f.starts_with("PREPEND") => {
					let bits = f.split("|").collect::<Vec<&str>>();
					if bits.len() != 2 {
						eprintln!(
							"OutputConfig error: Invalid PREPEND filter \"{filter}\"\n\
							Usage: PREPEND|[string]\n\
							Example:\n\
							cell1 = \"Bond\"\n\
							<cell1 PREPEND|'James '>\n\
							cell1 = \"James Bond\""
						);
						exit_with_error(1);
					}
					filters.push(Filter::Prepend(bits[1].to_string()));
				},
				f if f.starts_with("SPLIT") => {
					let bits = f.split("|").collect::<Vec<&str>>();
					if bits.len() != 3 {
						eprintln!(
							"OutputConfig error: Invalid SPLIT filter \"{filter}\"\n\
							Usage: SPLIT|[string]|[number]\n\
							Example:\n\
							cell1 = \"one,two,three,four\"\n\
							<cell1 SPLIT|','|3>\n\
							cell1 = \"two\""
						);
						exit_with_error(1);
					}
					let index = match bits[2].parse::<usize>() {
						Ok(n) => n,
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
							"OutputConfig error: Invalid SUB_STRING filter \"{filter}\"\n\
							Usage: SUB_STRING|[number]|[number optional]\n\
							Example:\n\
							cell1 = \"The Working Party\"\n\
							<cell1 SPLIT|4>\n\
							cell1 = \"Working Party\"\n\n\
							\
							cell1 = \"The Working Party\"\n\
							<cell1 SPLIT|4|7>\n\
							cell1 = \"Working\""
						);
						exit_with_error(1);
					}
					let start = match bits[1].parse::<usize>() {
						Ok(n) => n,
						Err(_) => {
							eprintln!(r#"OutputConfig error: Invalid SUB_STRING start "{}""#, bits[1]);
							exit_with_error(1);
						},
					};
					let end = if bits.len() == 3 {
						match bits[2].parse::<usize>() {
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
			Self::Length => Cow::Owned(input.len().to_string()),
			Self::Trim => Cow::Owned(input.trim().to_string()),
			Self::TrimStart => Cow::Owned(input.trim_start().to_string()),
			Self::TrimEnd => Cow::Owned(input.trim_end().to_string()),
			Self::Replace(search, replacement) => Cow::Owned(input.replace(search, replacement)),
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
			Self::Split(needle, index) => Cow::Owned(input.split(needle).nth(*index).unwrap_or_default().to_string()),
			Self::SubString(start, length) => {
				let start_byte = match input.char_indices().nth(*start) {
					Some((byte_idx, _)) => byte_idx,
					None => return Cow::Owned(String::from("")),
				};
				let end_byte = match *length {
					Some(len) => {
						if len == 0 {
							return Cow::Owned(String::from(""));
						} else {
							match input.char_indices().nth(start + len) {
								Some((byte_idx, _)) => byte_idx,
								None => input.len(),
							}
						}
					},
					None => input.len(),
				};
				Cow::Owned(input[start_byte..end_byte].to_string())
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
						eprintln!(r#"OutputConfig error: Cell number must be positive for item "{input}""#);
						exit_with_error(1);
					}
				},
				Err(_) => {
					eprintln!(r#"OutputConfig error: Invalid cell number "{input}""#);
					exit_with_error(1);
				},
			}
		} else if let Some(condition) = input.strip_prefix(":IF ") {
			let (condition, else_condition) = Condition::parse(condition);
			Item::If(condition, else_condition)
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
	pub fn new<R: BufRead>(config_file: CsvParser<R>) -> Self {
		let mut heading = String::new();
		let mut is_heading = true;
		let mut lines = Vec::new();

		for row in config_file {
			if is_heading {
				csv::export(&[row], &mut heading);
				heading.drain(..heading.len().saturating_sub(heading.trim_start().len()));
				heading.truncate(heading.trim_end().len());
				is_heading = false;
			} else {
				let mut cells = Vec::new();
				for cell in row {
					cells.push(Item::parse(cell.to_string()));
				}
				lines.push(cells);
			}
		}

		Self { heading, lines }
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::io::Cursor;

	#[test]
	fn new_test() {
		assert_eq!(
			OutputConfig::new(CsvParser::new(Cursor::new("heading1,heading2,heading3\n<cell1>,<cell2>,<cell3>\n"))),
			OutputConfig {
				heading: String::from("heading1,heading2,heading3"),
				lines: vec![vec![Item::Cell(0, None), Item::Cell(1, None), Item::Cell(2, None),]],
			}
		);

		assert_eq!(
			OutputConfig::new(CsvParser::new(Cursor::new("h1,h2,h3,h4\n<cell1>,,hardcoded,:IF my condition\n"))),
			OutputConfig {
				heading: String::from("h1,h2,h3,h4"),
				lines: vec![vec![
					Item::Cell(0, None),
					Item::Value(String::from("")),
					Item::Value(String::from("hardcoded")),
					Item::If(Condition::IsEmpty(Box::new(Item::Value(String::from("")))), None),
				]],
			}
		);
	}

	#[test]
	fn filter_test() {
		assert_eq!(
			OutputConfig::new(CsvParser::new(Cursor::new(
				"H1,H2,H3\n<cell1 LENGTH>,<cell2 UPPER_CASE LOWER_CASE REPLACE|' '|'-' SPLIT|'-'|3 APPEND|'end'>,<cell3>\n"
			))),
			OutputConfig {
				heading: String::from("H1,H2,H3"),
				lines: vec![vec![
					Item::Cell(0, Some(vec![Filter::Length])),
					Item::Cell(
						1,
						Some(vec![
							Filter::UpperCase,
							Filter::LowerCase,
							Filter::Replace(String::from(" "), String::from("-")),
							Filter::Split(String::from("-"), 3),
							Filter::Append(String::from("end")),
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
			OutputConfig::new(CsvParser::new(Cursor::new("H1\n<cell1 UPPER_CASE>\n"))),
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
			OutputConfig::new(CsvParser::new(Cursor::new("H1\n<cell1 LOWER_CASE>\n"))),
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
			OutputConfig::new(CsvParser::new(Cursor::new("H1\n<cell1 LENGTH>\n"))),
			OutputConfig {
				heading: String::from("H1"),
				lines: vec![vec![Item::Cell(0, Some(vec![Filter::Length,])),]],
			}
		);

		assert_eq!(Filter::Length.run(Cow::Borrowed("test")), Cow::Borrowed("4"));
		assert_eq!(Filter::Length.run(Cow::Borrowed("123456789 ")), Cow::Borrowed("10"));
	}

	#[test]
	fn trim_test() {
		assert_eq!(
			OutputConfig::new(CsvParser::new(Cursor::new("H1\n<cell1 TRIM>\n"))),
			OutputConfig {
				heading: String::from("H1"),
				lines: vec![vec![Item::Cell(0, Some(vec![Filter::Trim,])),]],
			}
		);

		assert_eq!(Filter::Trim.run(Cow::Borrowed(" te st  ")), Cow::Borrowed("te st"));
		assert_eq!(Filter::Trim.run(Cow::Borrowed(" \n te  st  \n  ")), Cow::Borrowed("te  st"));
	}

	#[test]
	fn trim_start_test() {
		assert_eq!(
			OutputConfig::new(CsvParser::new(Cursor::new("H1\n<cell1 TRIM_START>\n"))),
			OutputConfig {
				heading: String::from("H1"),
				lines: vec![vec![Item::Cell(0, Some(vec![Filter::TrimStart,])),]],
			}
		);

		assert_eq!(Filter::TrimStart.run(Cow::Borrowed(" \n   te  st  ")), Cow::Borrowed("te  st  "));
		assert_eq!(Filter::TrimStart.run(Cow::Borrowed("  te  st  \n  ")), Cow::Borrowed("te  st  \n  "));
	}

	#[test]
	fn trim_end_test() {
		assert_eq!(
			OutputConfig::new(CsvParser::new(Cursor::new("H1\n<cell1 TRIM_END>\n"))),
			OutputConfig {
				heading: String::from("H1"),
				lines: vec![vec![Item::Cell(0, Some(vec![Filter::TrimEnd,])),]],
			}
		);

		assert_eq!(Filter::TrimEnd.run(Cow::Borrowed(" \n   te  st  ")), Cow::Borrowed(" \n   te  st"));
		assert_eq!(Filter::TrimEnd.run(Cow::Borrowed("  te  st  \n  ")), Cow::Borrowed("  te  st"));
	}

	#[test]
	fn replace_test() {
		assert_eq!(
			OutputConfig::new(CsvParser::new(Cursor::new("H1\n<cell1 REPLACE|'-'|' '>\n"))),
			OutputConfig {
				heading: String::from("H1"),
				lines: vec![vec![Item::Cell(
					0,
					Some(vec![Filter::Replace(String::from("-"), String::from(" ")),])
				),]],
			}
		);

		assert_eq!(
			OutputConfig::new(CsvParser::new(Cursor::new("H1\n<cell1 REPLACE|'...'|'##'>\n"))),
			OutputConfig {
				heading: String::from("H1"),
				lines: vec![vec![Item::Cell(
					0,
					Some(vec![Filter::Replace(String::from("..."), String::from("##")),])
				),]],
			}
		);

		assert_eq!(
			Filter::Replace(String::from("blue"), String::from("green")).run(Cow::Borrowed("The shirt is blue")),
			Cow::Borrowed("The shirt is green")
		);
		assert_eq!(
			Filter::Replace(String::from(" "), String::from("")).run(Cow::Borrowed(" The shirt  is blue")),
			Cow::Borrowed("Theshirtisblue")
		);
	}

	#[test]
	fn append_test() {
		assert_eq!(
			OutputConfig::new(CsvParser::new(Cursor::new("H1\n<cell1 APPEND|'end'>\n"))),
			OutputConfig {
				heading: String::from("H1"),
				lines: vec![vec![Item::Cell(0, Some(vec![Filter::Append(String::from("end")),])),]],
			}
		);

		assert_eq!(
			OutputConfig::new(CsvParser::new(Cursor::new("H1\n<cell1 APPEND|'###'>\n"))),
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
			OutputConfig::new(CsvParser::new(Cursor::new("H1\n<cell1 PREPEND|'front'>\n"))),
			OutputConfig {
				heading: String::from("H1"),
				lines: vec![vec![Item::Cell(0, Some(vec![Filter::Prepend(String::from("front")),])),]],
			}
		);

		assert_eq!(
			OutputConfig::new(CsvParser::new(Cursor::new("H1\n<cell1 PREPEND|'###'>\n"))),
			OutputConfig {
				heading: String::from("H1"),
				lines: vec![vec![Item::Cell(0, Some(vec![Filter::Prepend(String::from("###")),])),]],
			}
		);

		assert_eq!(Filter::Prepend(String::from("start-")).run(Cow::Borrowed("middle")), Cow::Borrowed("start-middle"));
		assert_eq!(Filter::Prepend(String::from("😬 -")).run(Cow::Borrowed("middle")), Cow::Borrowed("😬 -middle"));
	}

	#[test]
	fn split_test() {
		assert_eq!(
			OutputConfig::new(CsvParser::new(Cursor::new("H1\n<cell1 SPLIT|'-'|6>\n"))),
			OutputConfig {
				heading: String::from("H1"),
				lines: vec![vec![Item::Cell(0, Some(vec![Filter::Split(String::from("-"), 6),])),]],
			}
		);

		assert_eq!(
			OutputConfig::new(CsvParser::new(Cursor::new("H1\n<cell1 SPLIT|'###'|666>\n"))),
			OutputConfig {
				heading: String::from("H1"),
				lines: vec![vec![Item::Cell(
					0,
					Some(vec![Filter::Split(String::from("###"), 666),])
				),]],
			}
		);

		assert_eq!(Filter::Split(String::from("-"), 3).run(Cow::Borrowed("0-1-2-3-4-5")), Cow::Borrowed("3"));
		assert_eq!(Filter::Split(String::from("-"), 10).run(Cow::Borrowed("0-1-2-3-4-5")), Cow::Borrowed(""));
		assert_eq!(Filter::Split(String::from("-"), 10).run(Cow::Borrowed("no dashes")), Cow::Borrowed(""));
	}

	#[test]
	fn sub_string_test() {
		assert_eq!(
			OutputConfig::new(CsvParser::new(Cursor::new("H1\n<cell1 SUB_STRING|5>\n"))),
			OutputConfig {
				heading: String::from("H1"),
				lines: vec![vec![Item::Cell(0, Some(vec![Filter::SubString(5, None)])),]],
			}
		);

		assert_eq!(
			OutputConfig::new(CsvParser::new(Cursor::new("H1\n<cell1 SUB_STRING|999|666>\n"))),
			OutputConfig {
				heading: String::from("H1"),
				lines: vec![vec![Item::Cell(0, Some(vec![Filter::SubString(999, Some(666))])),]],
			}
		);

		assert_eq!(Filter::SubString(5, None).run(Cow::Borrowed("12345678910 end")), Cow::Borrowed("678910 end"));
		assert_eq!(Filter::SubString(5, Some(3)).run(Cow::Borrowed("12345678910 end")), Cow::Borrowed("678"));
	}

	#[test]
	fn item_parse_test() {
		assert_eq!(Item::parse(String::from("TEST")), Item::Value(String::from("TEST")));
		assert_eq!(Item::parse(String::from("<cell1>")), Item::Cell(0, None));
		assert_eq!(Item::parse(String::from("<cell999>")), Item::Cell(998, None));
		assert_eq!(Item::parse(String::from("<cell1 UPPER_CASE>")), Item::Cell(0, Some(vec![Filter::UpperCase])));
		assert_eq!(
			Item::parse(String::from("<cell1 REPLACE|'\"'|'\\'' LOWER_CASE>")),
			Item::Cell(
				0,
				Some(vec![
					Filter::Replace(String::from("\""), String::from("'")),
					Filter::LowerCase
				])
			)
		);
	}

	#[test]
	fn filter_parsing_test() {
		assert_eq!(Filter::parse("UPPER_CASE"), vec![Filter::UpperCase]);
		assert_eq!(Filter::parse("LOWER_CASE"), vec![Filter::LowerCase]);
		assert_eq!(Filter::parse("LENGTH"), vec![Filter::Length]);
		assert_eq!(Filter::parse("TRIM"), vec![Filter::Trim]);
		assert_eq!(Filter::parse("TRIM_START"), vec![Filter::TrimStart]);
		assert_eq!(Filter::parse("TRIM_END"), vec![Filter::TrimEnd]);
		assert_eq!(Filter::parse("REPLACE|' '|''"), vec![Filter::Replace(String::from(" "), String::from(""))]);
		assert_eq!(Filter::parse("APPEND|'x'"), vec![Filter::Append(String::from("x"))]);
		assert_eq!(Filter::parse("PREPEND|'x'"), vec![Filter::Prepend(String::from("x"))]);
		assert_eq!(Filter::parse("SPLIT|'x'|3"), vec![Filter::Split(String::from("x"), 3)]);
		assert_eq!(Filter::parse("SUB_STRING|5"), vec![Filter::SubString(5, None)]);
		assert_eq!(Filter::parse("SUB_STRING|5|10"), vec![Filter::SubString(5, Some(10))]);

		assert_eq!(
			Filter::parse("REPLACE|'\"'|'\\'' LOWER_CASE"),
			vec![
				Filter::Replace(String::from("\""), String::from("'")),
				Filter::LowerCase
			]
		);

		assert_eq!(
			Filter::parse("UPPER_CASE LOWER_CASE LENGTH TRIM TRIM_START TRIM_END REPLACE|'blue'|'green' APPEND|'x' PREPEND|'x' SPLIT|' '|666 SUB_STRING|5 SUB_STRING|5|10"),
			vec![
				Filter::UpperCase,
				Filter::LowerCase,
				Filter::Length,
				Filter::Trim,
				Filter::TrimStart,
				Filter::TrimEnd,
				Filter::Replace(String::from("blue"), String::from("green")),
				Filter::Append(String::from("x")),
				Filter::Prepend(String::from("x")),
				Filter::Split(String::from(" "), 666),
				Filter::SubString(5, None),
				Filter::SubString(5, Some(10))
			]
		);
	}

	#[test]
	fn condition_parse_test() {
		// assert_eq!();
	}
}
