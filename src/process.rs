use crate::{
	cli::exit_with_error,
	config::{Item, OutputConfig},
};

pub fn run(input_line: &[Vec<String>], output_config: &OutputConfig) -> Vec<Vec<String>> {
	let mut new_lines = Vec::new();
	for items in output_config.lines.clone() {
		let mut line: Vec<String> = Vec::new();
		for item in items {
			match item {
				Item::Cell(i) => match input_line[0].get(i) {
					Some(v) => line.push(v.clone()),
					None => {
						eprintln!("Process error: Cell not found '{item}'");
						eprintln!("{input_line:?}");
						exit_with_error(1);
					},
				},
				Item::If(_condition) => {},
				Item::Value(v) => line.push(v.clone()),
			}
		}
		new_lines.push(line);
	}

	new_lines
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn run_cell_test() {
		assert_eq!(
			run(
				&vec![vec![String::from("A"), String::from("B"), String::from("C")]],
				&OutputConfig {
					heading: String::new(),
					lines: vec![vec![Item::Cell(0), Item::Cell(2), Item::Cell(1)]],
				},
			),
			vec![vec![String::from("A"), String::from("C"), String::from("B")]]
		);
	}

	#[test]
	fn run_value_test() {
		assert_eq!(
			run(
				&vec![vec![String::from("A"), String::from("B"), String::from("C")]],
				&OutputConfig {
					heading: String::new(),
					lines: vec![vec![Item::Value(String::from("NEW")), Item::Cell(2), Item::Cell(1)]],
				},
			),
			vec![vec![String::from("NEW"), String::from("C"), String::from("B")]]
		);
	}

	#[test]
	fn run_multiple_lines_test() {
		assert_eq!(
			run(
				&vec![vec![String::from("A"), String::from("B"), String::from("C")]],
				&OutputConfig {
					heading: String::new(),
					lines: vec![
						vec![Item::Cell(0), Item::Cell(2), Item::Cell(1)],
						vec![Item::Cell(0), Item::Cell(1), Item::Cell(2)],
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
				&vec![vec![String::from("A"), String::from("B"), String::from("C")]],
				&OutputConfig {
					heading: String::new(),
					lines: vec![
						vec![Item::Cell(0), Item::Cell(2), Item::Cell(1)],
						vec![Item::Cell(2), Item::Value(String::from("MERGE")), Item::Cell(2)],
						vec![Item::Cell(1), Item::Cell(0), Item::Value(String::from("NEW"))],
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
}
