use std::{borrow::Cow, io::BufRead};

use crate::{
	cli::exit_with_error,
	csv::{self, CsvParser},
};

#[derive(Debug, PartialEq, Clone)]
pub enum Condition {
	IsEmpty(Box<Item>),
	IsNotEmpty(Box<Item>),
	IsNumeric(Box<Item>),
	StartesWith(String, Box<Item>),
	EndsWith(String, Box<Item>),
	Contains(String, Box<Item>),
	Equals(Box<Item>, Box<Item>),
	NotEquals(Box<Item>, Box<Item>),
	GreaterThan(i64, Box<Item>),
	LessThan(i64, Box<Item>),
	Modulo(i64, i64, Box<Item>),
	// TODO: date functions
}

impl Condition {
	pub fn parse(condition_str: &str) -> (Self, Box<Item>, Option<Box<Item>>) {
		let usage = "\n\
		The syntax of an IF condition is: :IF <cell[x]> [condition] ([then-item]) [ELSE ([else-item])]\n\
		Examples:\n\
		:IF <cell1> == 'blue' ('green')\n\
		:IF <cell1> == <cell42> (<cell2>)\n\
		:IF <cell1 UPPER_CASE> == 'blue' ('green')\n\
		:IF <cell1> == 'blue' ('green') ELSE ('red')";

		if !condition_str.starts_with("<cell") {
			eprintln!("Condition error: Condition must start with <cell> item, was \"{condition_str}\"{usage}");
			exit_with_error(1);
		}

		if !condition_str.ends_with(")") {
			eprintln!("Condition error: Condition must end with a then-item or an else-item, was \"{condition_str}\"{usage}");
			exit_with_error(1);
		}

		let mut condition = None;
		let mut condition_item = None;
		let mut then_item = None;
		let mut else_item = None;
		let mut in_quotes = false;
		let mut in_condition = false;
		let mut escaped = false;
		let mut buffer = String::new();

		for c in condition_str.chars() {
			match c {
				'\'' => {
					if !escaped {
						in_quotes = !in_quotes;
					} else {
						buffer.push(c);
						escaped = false;
					}
				},
				'\\' => {
					escaped = !escaped;
				},
				'>' => {
					buffer.push(c);
					if !in_quotes {
						if condition_item.is_none() {
							condition_item = Some(Item::parse(buffer.trim().to_string()));
							in_condition = true;
							buffer.clear();
						} else if then_item.is_none() {
							if !in_condition {
								then_item = Some(Item::parse(buffer.trim().to_string()));
								buffer.clear();
							}
						} else {
							if !in_condition {
								else_item = Some(Box::new(Item::parse(buffer.trim().to_string())));
							} else {
								buffer.push(c);
							}
							buffer.clear();
						}
					} else {
						escaped = false;
					}
				},
				'(' => {
					if !in_quotes {
						in_condition = false;
						if condition.is_none() {
							condition = Some(buffer.trim().to_string());
						}
						buffer.clear();
					} else {
						buffer.push(c);
					}
				},
				')' => {
					if !in_quotes {
						if then_item.is_none() {
							then_item = Some(Item::parse(buffer.trim().to_string()));
						} else if else_item.is_none() && !buffer.is_empty() {
							else_item = Some(Box::new(Item::parse(buffer.trim().to_string())));
						}
						buffer.clear();
					} else {
						buffer.push(c);
					}
				},
				_ => {
					escaped = false;
					buffer.push(c);
				},
			}
		}

		if condition.is_none() {
			eprintln!("Condition error: Condition not found, was \"{condition_str}\"{usage}");
			exit_with_error(1);
		}

		if condition_item.is_none() {
			eprintln!("Condition error: Condition item not found, was \"{condition_str}\"{usage}");
			exit_with_error(1);
		}

		if then_item.is_none() {
			eprintln!("Condition error: Then item not found, was \"{condition_str}\"{usage}");
			exit_with_error(1);
		}

		// == 'this item'
		// != 'this item'
		// > 42
		// < 42
		// % 2 = 0
		// STARTS_WITH|'beginning'
		// IS_EMPTY
		// TODO: parse condition string
		let condition = match condition.unwrap().as_str() {
			c if c.starts_with("IS_EMPTY") => Condition::IsEmpty(Box::new(condition_item.unwrap())),
			c if c.starts_with("IS_NOT_EMPTY") => Condition::IsNotEmpty(Box::new(condition_item.unwrap())),
			c if c.starts_with("IS_NUMERIC") => Condition::IsNumeric(Box::new(condition_item.unwrap())),
			c if c.starts_with("STARTS_WITH") => {
				let needle = c.splitn(2, '|').collect::<Vec<&str>>();
				Condition::StartesWith(needle[1].to_string(), Box::new(condition_item.unwrap()))
			},
			c if c.starts_with("ENDS_WITH") => {
				let needle = c.splitn(2, '|').collect::<Vec<&str>>();
				Condition::EndsWith(needle[1].to_string(), Box::new(condition_item.unwrap()))
			},
			c if c.starts_with("CONTAINS") => {
				let needle = c.splitn(2, '|').collect::<Vec<&str>>();
				Condition::Contains(needle[1].to_string(), Box::new(condition_item.unwrap()))
			},
			c if c.starts_with("==") => {
				let start = if &c[2..3] == " " { 3 } else { 2 };
				Condition::Equals(Box::new(Item::parse(c[start..c.len()].to_string())), Box::new(condition_item.unwrap()))
			},
			c if c.starts_with("!=") => {
				let start = if &c[2..3] == " " { 3 } else { 2 };
				Condition::NotEquals(Box::new(Item::parse(c[start..c.len()].to_string())), Box::new(condition_item.unwrap()))
			},
			c if c.starts_with(">") => Condition::GreaterThan(0, Box::new(condition_item.unwrap())),
			c if c.starts_with("<") => Condition::LessThan(0, Box::new(condition_item.unwrap())),
			c if c.starts_with("%") => Condition::Modulo(0, 0, Box::new(condition_item.unwrap())),
			c => {
				eprintln!("Condition error: If condition not recognized, was \"{c}\"{usage}");
				exit_with_error(1);
			},
		};

		(condition, Box::new(then_item.unwrap()), else_item)
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
		let mut buffer = String::new();
		let mut filters = Vec::new();

		for c in filter_str.trim().chars() {
			match c {
				'\'' => {
					if !escaped {
						in_quotes = !in_quotes;
					} else {
						buffer.push(c);
						escaped = false;
					}
				},
				'\\' => {
					escaped = !escaped;
				},
				' ' => {
					if !in_quotes {
						filters_str.push(buffer.clone());
						buffer.clear();
						escaped = false;
					} else {
						buffer.push(c);
					}
				},
				_ => {
					escaped = false;
					buffer.push(c);
				},
			}
		}
		filters_str.push(buffer.clone());

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
	If(Condition, Box<Item>, Option<Box<Item>>),
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
			let (condition, then_item, else_item) = Condition::parse(condition);
			Item::If(condition, then_item, else_item)
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
			OutputConfig::new(CsvParser::new(Cursor::new("h1,h2,h3,h4\n<cell1>,,hardcoded,:IF <cell1> IS_EMPTY ('foo')\n"))),
			OutputConfig {
				heading: String::from("h1,h2,h3,h4"),
				lines: vec![vec![
					Item::Cell(0, None),
					Item::Value(String::from("")),
					Item::Value(String::from("hardcoded")),
					Item::If(Condition::IsEmpty(Box::new(Item::Cell(0, None))), Box::new(Item::Value(String::from("foo"))), None),
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
		Condition::parse("<cell1> == 'this item' ('yay') ELSE (<cell2>)");
		// TODO: Fill out test
	}

	#[test]
	fn conditional_isempty_test() {
		assert_eq!(
			Condition::parse("<cell1> IS_EMPTY ('yay') ELSE (<cell2>)"),
			(
				Condition::IsEmpty(Box::new(Item::Cell(0, None))),
				Box::new(Item::Value(String::from("yay"))),
				Some(Box::new(Item::Cell(1, None)))
			)
		);

		assert_eq!(
			Condition::parse("<cell1> IS_EMPTY (<cell2>)"),
			(Condition::IsEmpty(Box::new(Item::Cell(0, None))), Box::new(Item::Cell(1, None)), None)
		);
	}

	#[test]
	fn conditional_isnotempty_test() {
		assert_eq!(
			Condition::parse("<cell1> IS_NOT_EMPTY ('yay') ELSE (<cell2>)"),
			(
				Condition::IsNotEmpty(Box::new(Item::Cell(0, None))),
				Box::new(Item::Value(String::from("yay"))),
				Some(Box::new(Item::Cell(1, None)))
			)
		);

		assert_eq!(
			Condition::parse("<cell1> IS_NOT_EMPTY (<cell2>)"),
			(Condition::IsNotEmpty(Box::new(Item::Cell(0, None))), Box::new(Item::Cell(1, None)), None)
		);
	}

	#[test]
	fn conditional_isnumeric_test() {
		assert_eq!(
			Condition::parse("<cell1> IS_NUMERIC ('yay') ELSE (<cell2>)"),
			(
				Condition::IsNumeric(Box::new(Item::Cell(0, None))),
				Box::new(Item::Value(String::from("yay"))),
				Some(Box::new(Item::Cell(1, None)))
			)
		);

		assert_eq!(
			Condition::parse("<cell1> IS_NUMERIC (<cell2>)"),
			(Condition::IsNumeric(Box::new(Item::Cell(0, None))), Box::new(Item::Cell(1, None)), None)
		);
	}

	#[test]
	fn conditional_starteswith_test() {
		assert_eq!(
			Condition::parse("<cell1> STARTS_WITH|'foo' ('ya\\'y') ELSE (<cell2>)"),
			(
				Condition::StartesWith(String::from("foo"), Box::new(Item::Cell(0, None))),
				Box::new(Item::Value(String::from("ya'y"))),
				Some(Box::new(Item::Cell(1, None)))
			)
		);

		assert_eq!(
			Condition::parse("<cell1> STARTS_WITH|'foo' (<cell2>)"),
			(Condition::StartesWith(String::from("foo"), Box::new(Item::Cell(0, None))), Box::new(Item::Cell(1, None)), None)
		);

		assert_eq!(
			Condition::parse("<cell1> STARTS_WITH|'fo\\'o' (<cell2>)"),
			(
				Condition::StartesWith(String::from("fo'o"), Box::new(Item::Cell(0, None))),
				Box::new(Item::Cell(1, None)),
				None
			)
		);
	}

	#[test]
	fn conditional_endswith_test() {
		assert_eq!(
			Condition::parse("<cell1> ENDS_WITH|'foo' ('ya\\'y') ELSE (<cell2>)"),
			(
				Condition::EndsWith(String::from("foo"), Box::new(Item::Cell(0, None))),
				Box::new(Item::Value(String::from("ya'y"))),
				Some(Box::new(Item::Cell(1, None)))
			)
		);

		assert_eq!(
			Condition::parse("<cell1> ENDS_WITH|'foo' (<cell2>)"),
			(Condition::EndsWith(String::from("foo"), Box::new(Item::Cell(0, None))), Box::new(Item::Cell(1, None)), None)
		);

		assert_eq!(
			Condition::parse("<cell1> ENDS_WITH|'fo\\'o' (<cell2>)"),
			(Condition::EndsWith(String::from("fo'o"), Box::new(Item::Cell(0, None))), Box::new(Item::Cell(1, None)), None)
		);
	}

	#[test]
	fn conditional_contains_test() {
		assert_eq!(
			Condition::parse("<cell1> CONTAINS|'foo' ('yay') ELSE (<cell2>)"),
			(
				Condition::Contains(String::from("foo"), Box::new(Item::Cell(0, None))),
				Box::new(Item::Value(String::from("yay"))),
				Some(Box::new(Item::Cell(1, None)))
			)
		);

		assert_eq!(
			Condition::parse("<cell1> CONTAINS|'foo' (<cell2>)"),
			(Condition::Contains(String::from("foo"), Box::new(Item::Cell(0, None))), Box::new(Item::Cell(1, None)), None)
		);
	}

	#[test]
	fn conditional_equals_test() {
		assert_eq!(
			Condition::parse("<cell1> == 'foo' ('yay') ELSE (<cell2>)"),
			(
				Condition::Equals(Box::new(Item::Value(String::from("foo"))), Box::new(Item::Cell(0, None))),
				Box::new(Item::Value(String::from("yay"))),
				Some(Box::new(Item::Cell(1, None)))
			)
		);

		assert_eq!(
			Condition::parse("<cell1> == 'foo' (<cell2>)"),
			(
				Condition::Equals(Box::new(Item::Value(String::from("foo"))), Box::new(Item::Cell(0, None))),
				Box::new(Item::Cell(1, None)),
				None
			)
		);

		assert_eq!(
			Condition::parse("<cell1>=='foo'(<cell2>)"),
			(
				Condition::Equals(Box::new(Item::Value(String::from("foo"))), Box::new(Item::Cell(0, None))),
				Box::new(Item::Cell(1, None)),
				None
			)
		);

		assert_eq!(
			Condition::parse("<cell1> == <cell666> (<cell2>)"),
			(
				Condition::Equals(Box::new(Item::Cell(665, None)), Box::new(Item::Cell(0, None))),
				Box::new(Item::Cell(1, None)),
				None
			)
		);
	}

	#[test]
	fn conditional_notequals_test() {
		assert_eq!(
			Condition::parse("<cell1> != 'foo' ('yay') ELSE (<cell2>)"),
			(
				Condition::NotEquals(Box::new(Item::Value(String::from("foo"))), Box::new(Item::Cell(0, None))),
				Box::new(Item::Value(String::from("yay"))),
				Some(Box::new(Item::Cell(1, None)))
			)
		);

		assert_eq!(
			Condition::parse("<cell1> != 'foo' (<cell2>)"),
			(
				Condition::NotEquals(Box::new(Item::Value(String::from("foo"))), Box::new(Item::Cell(0, None))),
				Box::new(Item::Cell(1, None)),
				None
			)
		);

		assert_eq!(
			Condition::parse("<cell1>!='foo'(<cell2>)"),
			(
				Condition::NotEquals(Box::new(Item::Value(String::from("foo"))), Box::new(Item::Cell(0, None))),
				Box::new(Item::Cell(1, None)),
				None
			)
		);

		assert_eq!(
			Condition::parse("<cell1> != <cell666> (<cell2>)"),
			(
				Condition::NotEquals(Box::new(Item::Cell(665, None)), Box::new(Item::Cell(0, None))),
				Box::new(Item::Cell(1, None)),
				None
			)
		);
	}

	#[test]
	fn conditional_greaterthan_test() {}

	#[test]
	fn conditional_lessthan_test() {}

	#[test]
	fn conditional_modulo_test() {}
}
