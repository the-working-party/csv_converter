#[derive(Debug, Default, PartialEq)]
pub struct Settings {
	pub input: String,
	pub output: String,
	pub config: String,
	pub version: bool,
	pub help: bool,
}

impl Settings {
	pub fn new(args: Vec<String>) -> Self {
		let mut settings: Settings = Default::default();

		let mut args_iter = args.into_iter();
		while let Some(arg) = args_iter.next() {
			match arg.as_str() {
				"-i" | "--input" => {
					let item = args_iter.next();
					if item.is_none() {
						eprintln!("Error: Expected an argument after '{arg}'");
						exit_with_error(1);
					} else {
						settings.input = item.unwrap();
					}
				},
				"-o" | "--output" => {
					let item = args_iter.next();
					if item.is_none() {
						eprintln!("Error: Expected an argument after '{arg}'");
						exit_with_error(1);
					} else {
						settings.output = item.unwrap();
					}
				},
				"-c" | "--config" => {
					let item = args_iter.next();
					if item.is_none() {
						eprintln!("Error: Expected an argument after '{arg}'");
						exit_with_error(1);
					} else {
						settings.config = item.unwrap();
					}
				},
				"-v" | "-V" | "--version" => {
					settings.version = true;
				},
				"-h" | "--help" => {
					settings.help = true;
				},
				_ => {},
			}
		}

		if settings.input.is_empty() && !settings.version && !settings.help {
			eprintln!("Error: Missing parameter 'input'");
			println!("{}", usage());
			exit_with_error(1);
		}

		if settings.output.is_empty() && !settings.version && !settings.help {
			eprintln!("Error: Missing parameter 'output'");
			println!("{}", usage());
			exit_with_error(1);
		}

		if settings.config.is_empty() && !settings.version && !settings.help {
			eprintln!("Error: Missing parameter 'config'");
			println!("{}", usage());
			exit_with_error(1);
		}

		settings
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parsing_args_shortcut_test() {
		assert_eq!(
			Settings::new(vec![
				String::from("-i"),
				String::from("input_file.csv"),
				String::from("-o"),
				String::from("output_file.csv"),
				String::from("-c"),
				String::from("config_file.csv"),
			]),
			Settings {
				input: String::from("input_file.csv"),
				output: String::from("output_file.csv"),
				config: String::from("config_file.csv"),
				version: false,
				help: false,
			}
		);
	}

	#[test]
	fn parsing_args_longform_test() {
		assert_eq!(
			Settings::new(vec![
				String::from("--input"),
				String::from("input_file.csv"),
				String::from("--output"),
				String::from("output_file.csv"),
				String::from("--config"),
				String::from("config_file.csv"),
			]),
			Settings {
				input: String::from("input_file.csv"),
				output: String::from("output_file.csv"),
				config: String::from("config_file.csv"),
				version: false,
				help: false,
			}
		);
	}

	#[test]
	#[should_panic]
	fn missing_input_shortcut_test() {
		Settings::new(vec![
			String::from("-o"),
			String::from("output_file.csv"),
			String::from("-c"),
			String::from("config_file.csv"),
		]);
	}

	#[test]
	#[should_panic]
	fn missing_input_longform_test() {
		Settings::new(vec![
			String::from("--output"),
			String::from("output_file.csv"),
			String::from("--config"),
			String::from("config_file.csv"),
		]);
	}

	#[test]
	#[should_panic]
	fn missing_output_shortcut_test() {
		Settings::new(vec![
			String::from("-i"),
			String::from("input_file.csv"),
			String::from("-c"),
			String::from("config_file.csv"),
		]);
	}

	#[test]
	#[should_panic]
	fn missing_output_longform_test() {
		Settings::new(vec![
			String::from("--input"),
			String::from("input_file.csv"),
			String::from("--config"),
			String::from("config_file.csv"),
		]);
	}

	#[test]
	#[should_panic]
	fn missing_config_shortcut_test() {
		Settings::new(vec![
			String::from("-i"),
			String::from("input_file.csv"),
			String::from("-o"),
			String::from("output_file.csv"),
		]);
	}

	#[test]
	#[should_panic]
	fn missing_all_test() {
		Settings::new(Vec::new());
	}
}

pub fn help() -> String {
	format!(
		r#"
 █▀▀ █▀▀ █ █   ▀█▀ █▀█   █▀▄▀█ ▄▀█ ▀█▀ █▀█ █ ▀▄▀ █ █▀▀ █▄█
 █▄▄ ▄▄█ ▀▄▀    █  █▄█   █ ▀ █ █▀█  █  █▀▄ █ █ █ █ █▀   █
A tool to build a matrixify compatible CSV
{}"#,
		usage()
	)
}

fn usage() -> String {
	format!(
		r#"
Usage: {} [OPTIONS]

Options:
  -i <file>, --input <file>
        Specify the input file to process.
  -o <file>, --output <file>
        Specify the output file to write results to.
  -c <file>, --config <file>
        Specify the config file to determin what the output format is.
  -v, -V, --version
        Display the program's version information.
  -h, --help
        Display this help message."#,
		env!("CARGO_PKG_NAME")
	)
}

pub fn exit_with_error(code: i32) -> ! {
	if cfg!(test) {
		panic!("Process would exit with code: {}", code);
	} else {
		std::process::exit(code);
	}
}
