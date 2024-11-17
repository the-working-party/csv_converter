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
	_UpperCase,
	_LowerCase,
	_Split(String),
	_SubString(u64, Option<u64>),
	_Replace(String),
	_Append(String),
	_Preppend(String),
	Length,
	_Trim,
	_TrimStart,
	_TrimEnd,
}

impl Filter {
	pub fn parse<'a>(_filter: &'a str) -> Vec<Self> {
		vec![Self::Length]
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
			OutputConfig::new("H1,H2,H3\n<cell1>,<cell2 UPPER_CASE>,<cell3>\n"),
			OutputConfig {
				heading: String::from("H1,H2,H3"),
				lines: vec![vec![
					Item::Cell(0, None),
					Item::Cell(1, Some(vec![Filter::Length])),
					Item::Cell(2, None),
				]],
			}
		);
	}
}
