//! This module handles:
//! - Parsing CLI arguments
//! - A function to display help
//! - A function to display color in the terminal
//! - A function to handle exiting with the right error code
use CliColor::*;

/// The [Settings] struct is the code representation of your CI arguments
#[derive(Debug, Default, PartialEq)]
pub struct Settings {
	/// The path for the input file
	pub input: String,
	/// The path for the output file
	pub output: String,
	/// The path for the config file
	pub output_config: String,
	/// A boolean flag to show the version
	pub version: bool,
	/// A boolean flag to show the help
	pub help: bool,
}

impl Settings {
	/// This function parses the CLI arguments into our [Settings] struct.
	/// Pass in the args from env: `Settings::new(env::args().skip(1).collect());`
	///
	/// ```rust
	/// use csv_converter::cli::Settings;
	///
	/// assert_eq!(
	///     Settings::new(vec![
	///         String::from("-i"),
	///         String::from("input_file.csv"),
	///         String::from("-o"),
	///         String::from("output_file.csv"),
	///         String::from("-c"),
	///         String::from("config_file.csv"),
	///     ]),
	///     Settings {
	///         input: String::from("input_file.csv"),
	///         output: String::from("output_file.csv"),
	///         output_config: String::from("config_file.csv"),
	///         version: false,
	///         help: false,
	///     }
	/// );
	/// ```
	pub fn new(args: Vec<String>) -> Self {
		let mut settings: Settings = Default::default();

		let mut args_iter = args.into_iter();
		while let Some(arg) = args_iter.next() {
			match arg.as_str() {
				"-i" | "--input" => {
					let item = args_iter.next();
					if item.is_none() {
						exit_with_error(Some(format!("Expected an argument after \"{arg}\"")), Some(ErrorStages::Cli), 1);
					} else {
						settings.input = item.unwrap();
					}
				},
				"-o" | "--output" => {
					let item = args_iter.next();
					if item.is_none() {
						exit_with_error(Some(format!("Expected an argument after \"{arg}\"")), Some(ErrorStages::Cli), 1);
					} else {
						settings.output = item.unwrap();
					}
				},
				"-c" | "--config" => {
					let item = args_iter.next();
					if item.is_none() {
						exit_with_error(Some(format!("Expected an argument after \"{arg}\"")), Some(ErrorStages::Cli), 1);
					} else {
						settings.output_config = item.unwrap();
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
			exit_with_error(Some(format!("Missing parameter  \"input\"\n{}", usage())), Some(ErrorStages::Cli), 1);
		}

		if settings.output.is_empty() && !settings.version && !settings.help {
			exit_with_error(Some(format!("Missing parameter  \"output\"\n{}", usage())), Some(ErrorStages::Cli), 1);
		}

		if settings.output_config.is_empty() && !settings.version && !settings.help {
			exit_with_error(Some(format!("Missing parameter  \"config\"\n{}", usage())), Some(ErrorStages::Cli), 1);
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
				output_config: String::from("config_file.csv"),
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
				output_config: String::from("config_file.csv"),
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

/// This is a simple function to display the help within the terminal
pub fn help() -> String {
	format!(
		r#"
{YellowBright } █▀▀ █▀▀ █ █   █▀▀ █▀█ █▄ █ █ █ █▀▀ █▀█ ▀█▀ █▀▀ █▀█{Reset}
{MagentaBright} █▄▄ ▄▄█ ▀▄▀   █▄▄ █▄█ █ ▀█ ▀▄▀ ██▄ █▀▄  █  ██▄ █▀▄{Reset}
 A tool to convert a CSV file into a new format
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
        Specify the config file to determine what the output format is.
  -v, -V, --version
        Display the program's version information.
  -h, --help
        Display this help message."#,
		env!("CARGO_PKG_NAME")
	)
}

/// The stages of errors possible
pub enum ErrorStages {
	/// Error in the CLI module
	Cli,
	/// Error in the config parser
	ConfigParsing,
	/// Error in the config parser while parsing conditions
	ConfigConditionParsing,
	/// Error in the config parser while evaluating conditions
	ConfigConditionEvaluating,
	/// Error in the config parser while parsing filters
	ConfigFilterParsing,
	/// Error in the process module
	Process,
	/// Errors while ding I/O
	Io,
}

/// Handle exist gracefully and display a nice message
pub fn exit_with_error(error: Option<String>, stage: Option<ErrorStages>, code: i32) -> ! {
	if error.is_some() && stage.is_some() {
		let prefix = match stage.unwrap() {
			ErrorStages::Cli => format!("{Yellow}CLI{Reset}:"),
			ErrorStages::ConfigParsing => format!("{Yellow}Config{Reset}::{Yellow}Parsing{Reset}:"),
			ErrorStages::ConfigConditionParsing => {
				format!("{Yellow}Config{Reset}::{Yellow}Condition{Reset}::{Yellow}Parsing{Reset}:")
			},
			ErrorStages::ConfigConditionEvaluating => {
				format!("{Yellow}Config{Reset}::{Yellow}Condition{Reset}::{Yellow}Evaluating{Reset}:")
			},
			ErrorStages::ConfigFilterParsing => {
				format!("{Yellow}Config{Reset}::{Yellow}Filter{Reset}::{Yellow}Parsing{Reset}:")
			},
			ErrorStages::Process => format!("{Yellow}Processing{Reset}:"),
			ErrorStages::Io => format!("{Yellow}I/O{Reset}:"),
		};
		eprintln!(" {Red}ERROR{Reset} {Yellow}{prefix}{Reset} {}", error.unwrap());
	}

	if cfg!(test) {
		panic!("error=\"{}\" code=\"{code}\"", code);
	} else {
		std::process::exit(code);
	}
}

/// Terminal color ANSI codes
/// ```rust
/// use csv_converter::cli::CliColor::*;
///
/// assert_eq!(
///     format!("{Red}Hello World{Reset}"),
///     String::from("\x1b[31mHello World\x1b[39m"),
/// );
/// ```
#[allow(dead_code)]
pub enum CliColor {
	System,
	Black,
	Red,
	Green,
	Yellow,
	Blue,
	Magenta,
	Cyan,
	White,
	Gray,
	RedBright,
	GreenBright,
	YellowBright,
	BlueBright,
	MagentaBright,
	CyanBright,
	WhiteBright,
	Reset,
}

impl std::fmt::Display for CliColor {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			CliColor::System => write!(f, "\x1b[39m"),
			CliColor::Black => write!(f, "\x1b[30m"),
			CliColor::Red => write!(f, "\x1b[31m"),
			CliColor::Green => write!(f, "\x1b[32m"),
			CliColor::Yellow => write!(f, "\x1b[33m"),
			CliColor::Blue => write!(f, "\x1b[34m"),
			CliColor::Magenta => write!(f, "\x1b[35m"),
			CliColor::Cyan => write!(f, "\x1b[36m"),
			CliColor::White => write!(f, "\x1b[37m"),
			CliColor::Gray => write!(f, "\x1b[90m"),
			CliColor::RedBright => write!(f, "\x1b[91m"),
			CliColor::GreenBright => write!(f, "\x1b[92m"),
			CliColor::YellowBright => write!(f, "\x1b[93m"),
			CliColor::BlueBright => write!(f, "\x1b[94m"),
			CliColor::MagentaBright => write!(f, "\x1b[95m"),
			CliColor::CyanBright => write!(f, "\x1b[96m"),
			CliColor::WhiteBright => write!(f, "\x1b[97m"),
			CliColor::Reset => write!(f, "\x1b[39m"),
		}
	}
}
