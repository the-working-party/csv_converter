use std::{
	env,
	fs::File,
	io::{BufReader, BufWriter, Write},
	time::{Duration, Instant},
};

mod cli;
mod config;
mod csv;
mod process;

use crate::{
	cli::{exit_with_error, help, CliColor::*, ErrorStages, Settings},
	config::OutputConfig,
	csv::CsvParser,
};

fn main() {
	let time = Instant::now();
	let settings = Settings::new(env::args().skip(1).collect());

	if settings.version {
		println!("v{}", env!("CARGO_PKG_VERSION"));
		exit_with_error(None, None, 0);
	}

	if settings.help {
		println!("{}", help());
		exit_with_error(None, None, 0);
	}

	let output_config = match File::open(&settings.output_config) {
		Ok(file) => {
			let reader = BufReader::new(file);
			let config_file = CsvParser::new(reader);
			OutputConfig::new(config_file)
		},
		Err(error) => {
			exit_with_error(
				Some(format!("Could not open output config \"{}\": \"{Red}{error}{Reset}\"", settings.output)),
				Some(ErrorStages::Io),
				1,
			);
		},
	};

	let input_file = match File::open(&settings.input) {
		Ok(file) => file,
		Err(error) => {
			exit_with_error(
				Some(format!("Could not open input file \"{}\": \"{Red}{error}{Reset}\"", settings.output)),
				Some(ErrorStages::Io),
				1,
			);
		},
	};
	let total_size = match input_file.metadata() {
		Ok(metadata) => metadata.len(),
		Err(error) => {
			exit_with_error(
				Some(format!("Could not get metadata for input file \"{}\": \"{Red}{error}{Reset}\"", settings.output)),
				Some(ErrorStages::Io),
				1,
			);
		},
	};
	let reader = BufReader::with_capacity(64 * 1024, input_file);

	let output_file = match File::create(&settings.output) {
		Ok(file) => file,
		Err(error) => {
			exit_with_error(
				Some(format!("Could not create output file \"{}\": \"{Red}{error}{Reset}\"", settings.output)),
				Some(ErrorStages::Io),
				1,
			);
		},
	};
	let mut writer = BufWriter::with_capacity(256 * 1024, output_file);

	let mut is_heading = true;
	let mut output = String::new();
	let mut last_report_time = Instant::now();

	let mut csv_file = CsvParser::new(reader);

	println!("Progress: 0%");
	while let Some(row) = csv_file.next() {
		if is_heading {
			is_heading = false;
			output = format!("{}\n", output_config.heading);
		} else {
			csv::export(&process::run(&row, &output_config), &mut output);
		};

		if let Err(error) = writer.write_all(output.as_bytes()) {
			exit_with_error(
				Some(format!("Failed to write to output file: \"{Red}{error}{Reset}\"")),
				Some(ErrorStages::Io),
				1,
			);
		}

		if last_report_time.elapsed() >= Duration::from_secs(1) {
			let progress = (csv_file.bytes_read as f64 / total_size as f64) * 100.0;
			println!("\x1b[1A\x1b[0GProgress: {:.2}%\x1b[0K", progress);
			last_report_time = Instant::now();
		}
	}
	print!("\x1b[1A\x1b[0G");

	if let Err(error) = writer.flush() {
		exit_with_error(Some(format!("Failed to flush output file: \"{Red}{error}{Reset}\"")), Some(ErrorStages::Io), 1);
	} else {
		println!("File successfully written to \"{GreenBright}{}{Reset}\"\nTime: {:#?}", settings.output, time.elapsed())
	}
}
