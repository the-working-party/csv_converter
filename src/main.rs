use std::{
	env,
	fs::{read_to_string, File},
	io::{BufReader, BufWriter, Write},
	time::{Duration, Instant},
};

mod cli;
mod config;
mod csv;
mod process;

use crate::{
	cli::{exit_with_error, help, Settings},
	config::OutputConfig,
	csv::CsvLine,
};

fn main() {
	let time = Instant::now();
	let settings = Settings::new(env::args().skip(1).collect());

	if settings.version {
		println!("v{}", env!("CARGO_PKG_VERSION"));
		exit_with_error(0);
	}

	if settings.help {
		println!("{}", help());
		exit_with_error(0);
	}

	let output_config = match read_to_string(&settings.output_config) {
		Ok(contents) => OutputConfig::new(&contents),
		Err(error) => {
			eprintln!("Error: Could not create output file '{}': {error}", settings.output);
			exit_with_error(1);
		},
	};

	let input_file = match File::open(&settings.input) {
		Ok(file) => file,
		Err(error) => {
			eprintln!("Error: Could not open input file '{}': {error}", settings.input);
			exit_with_error(1);
		},
	};
	let total_size = match input_file.metadata() {
		Ok(metadata) => metadata.len(),
		Err(error) => {
			eprintln!("Error: Could not get metadata for input file '{}': {error}", settings.input);
			exit_with_error(1);
		},
	};
	let mut reader = BufReader::new(input_file);

	let output_file = match File::create(&settings.output) {
		Ok(file) => file,
		Err(error) => {
			eprintln!("Error: Could not create output file '{}': {error}", settings.output);
			exit_with_error(1);
		},
	};
	let mut writer = BufWriter::new(output_file);

	let mut line = String::new();
	let mut temp_line = String::new();
	let mut is_heading = true;
	let mut csv = CsvLine::new(output_config);
	let mut bytes_read: u128 = 0;
	let mut last_report_time = Instant::now();

	println!("Progress: 0%");
	loop {
		if !CsvLine::read_csv_record(&mut reader, &mut line, &mut temp_line, &mut bytes_read, &settings) {
			break;
		}

		if let Err(error) = writer.write_all(csv.parse_line(line.trim(), is_heading).as_bytes()) {
			eprintln!("Error: Failed to write to output file: {error}");
			exit_with_error(1);
		}

		if last_report_time.elapsed() >= Duration::from_secs(1) {
			let progress = (bytes_read as f64 / total_size as f64) * 100.0;
			println!("\x1b[1A\x1b[0GProgress: {:.2}%\x1b[0K", progress);
			last_report_time = Instant::now();
		}

		is_heading = false;
	}
	print!("\x1b[1A\x1b[0G");

	if let Err(error) = writer.flush() {
		eprintln!("Error: Failed to flush output file: {}", error);
		exit_with_error(1);
	} else {
		println!("File successfully written to {:?}\nTime: {:#?}", settings.output, time.elapsed())
	}
}
