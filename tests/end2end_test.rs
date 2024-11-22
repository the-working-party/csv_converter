use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn test_csv_converter_end_to_end() {
	let input_file = "tests/input.csv";
	let output_file = "tests/output.csv";
	let config_file = "tests/config.csv";
	let expected_output_file = "tests/expected_output.csv";

	if Path::new(output_file).exists() {
		fs::remove_file(output_file).expect("Failed to remove old output file");
	}

	let output = Command::new(env!("CARGO_BIN_EXE_csv_converter"))
		.arg("-i")
		.arg(input_file)
		.arg("-o")
		.arg(output_file)
		.arg("-c")
		.arg(config_file)
		.output()
		.expect("Failed to execute csv_converter");

	assert!(
		output.status.success(),
		"csv_converter did not run successfully: {}",
		String::from_utf8_lossy(&output.stderr)
	);

	assert!(Path::new(output_file).exists(), "Output file was not created");

	let actual_output = fs::read_to_string(output_file).expect("Failed to read the output file");
	let expected_output = fs::read_to_string(expected_output_file).expect("Failed to read the expected output file");

	assert_eq!(actual_output, expected_output, "The output does not match the expected output");
}
