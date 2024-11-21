use std::borrow::Cow;

use crate::{
	cli::exit_with_error,
	config::{Item, OutputConfig},
};

pub fn run(input_line: &Vec<String>, output_config: &OutputConfig) -> Vec<Vec<String>> {
	let mut new_lines = Vec::new();
	let mut skip_line = false;
	for items in &output_config.lines {
		let mut line: Vec<String> = Vec::new();
		for item in items {
			match item {
				Item::Cell(i, filters) => match input_line.get(*i) {
					Some(v) => {
						let mut value: Cow<str> = Cow::Borrowed(v.as_str());
						if let Some(filters) = filters {
							for filter in filters {
								value = filter.run(value);
							}
						}
						line.push(value.to_string())
					},
					None => {
						eprintln!("Process error: Cell not found '<cell{i}>'");
						eprintln!("{input_line:?}");
						exit_with_error(1);
					},
				},
				Item::If(condition, then_item, else_item) => {
					let condition_result =
						condition.run(then_item, &else_item.as_ref().map(|b| (**b).clone()), input_line).to_string();
					if &condition_result == "SKIP_LINE" {
						skip_line = true;
					}
					line.push(condition_result)
				},
				Item::Value(v) => line.push(v.clone()),
			}
		}
		if skip_line {
			skip_line = false;
		} else {
			new_lines.push(line);
		}
	}

	new_lines
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::csv::CsvParser;
	use std::io::Cursor;

	#[test]
	fn run_cell_test() {
		assert_eq!(
			run(
				&vec![String::from("A"), String::from("B"), String::from("C")],
				&OutputConfig::new(CsvParser::new(Cursor::new("A,B,C\n<cell1>,<cell3>,<cell2>\n"))),
			),
			vec![vec![String::from("A"), String::from("C"), String::from("B")]]
		);
	}

	#[test]
	fn run_value_test() {
		assert_eq!(
			run(
				&vec![String::from("A"), String::from("B"), String::from("C")],
				&OutputConfig {
					heading: String::new(),
					lines: vec![vec![
						Item::Value(String::from("NEW")),
						Item::Cell(2, None),
						Item::Cell(1, None)
					]],
				},
			),
			vec![vec![String::from("NEW"), String::from("C"), String::from("B")]]
		);
	}

	#[test]
	fn run_multiple_lines_test() {
		assert_eq!(
			run(
				&vec![String::from("A"), String::from("B"), String::from("C")],
				&OutputConfig {
					heading: String::new(),
					lines: vec![
						vec![Item::Cell(0, None), Item::Cell(2, None), Item::Cell(1, None)],
						vec![Item::Cell(0, None), Item::Cell(1, None), Item::Cell(2, None)],
					],
				},
			),
			vec![
				vec![String::from("A"), String::from("C"), String::from("B")],
				vec![String::from("A"), String::from("B"), String::from("C")],
			]
		);
	}

	#[test]
	fn run_everything_test() {
		assert_eq!(
			run(
				&vec![String::from("A"), String::from("B"), String::from("C")],
				&OutputConfig {
					heading: String::new(),
					lines: vec![
						vec![Item::Cell(0, None), Item::Cell(2, None), Item::Cell(1, None)],
						vec![
							Item::Cell(2, None),
							Item::Value(String::from("MERGE")),
							Item::Cell(2, None)
						],
						vec![
							Item::Cell(1, None),
							Item::Cell(0, None),
							Item::Value(String::from("NEW"))
						],
					],
				},
			),
			vec![
				vec![String::from("A"), String::from("C"), String::from("B")],
				vec![String::from("C"), String::from("MERGE"), String::from("C")],
				vec![String::from("B"), String::from("A"), String::from("NEW")],
			]
		);
	}

	#[test]
	fn skip_line_test() {
		assert_eq!(
			run(
				&vec![
					String::from("A"),
					String::from("B"),
					String::from("C"),
					String::from("D")
				],
				&OutputConfig::new(CsvParser::new(Cursor::new("Column A,Column B,Column C\n<cell1>,MERGE,<cell2>\n<cell1>,NEW,:IF <cell3> == 'D' ('SKIP_LINE') ELSE (<cell3>)\n<cell1>,NEW,:IF <cell4> == 'D' ('SKIP_LINE') ELSE (<cell4>)\n"))),
			),
			vec![
				vec![String::from("A"), String::from("MERGE"), String::from("B")],
				vec![String::from("A"), String::from("NEW"), String::from("C")],
			]
		);
	}
}
