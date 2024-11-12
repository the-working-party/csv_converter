use std::{
	env,
	fs::File,
	io::{BufRead, BufReader, BufWriter, Write},
};

mod cli;
mod csv;

use cli::{exit_with_error, help, Settings};
use csv::CsvLine;

fn main() {
	let settings = Settings::new(env::args().skip(1).collect());

	if settings.version {
		println!("v{}", env!("CARGO_PKG_VERSION"));
		exit_with_error(0);
	}

	if settings.help {
		println!("{}", help());
		exit_with_error(0);
	}

	let input_file = match File::open(&settings.input) {
		Ok(file) => file,
		Err(error) => {
			eprintln!("Error: Could not open input file '{}': {}", settings.input, error);
			exit_with_error(1);
		},
	};
	let mut reader = BufReader::new(input_file);

	let output_file = match File::create(&settings.output) {
		Ok(file) => file,
		Err(error) => {
			eprintln!("Error: Could not create output file '{}': {}", settings.output, error);
			exit_with_error(1);
		},
	};
	let mut writer = BufWriter::new(output_file);

	let mut line = String::new();

	loop {
		line.clear();
		let bytes_read = match reader.read_line(&mut line) {
			Ok(bytes) => bytes,
			Err(error) => {
				eprintln!("Error: Failed to read from input file: {}", error);
				exit_with_error(1);
			},
		};

		if bytes_read == 0 {
			break;
		}

		if let Err(error) = writer.write_all(CsvLine::new(line.trim()).process().export().as_bytes()) {
			eprintln!("Error: Failed to write to output file: {}", error);
			exit_with_error(1);
		}
	}

	if let Err(error) = writer.flush() {
		eprintln!("Error: Failed to flush output file: {}", error);
		exit_with_error(1);
	}
}
