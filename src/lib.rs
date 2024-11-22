//! ```skip
//!  █▀▀ █▀▀ █ █   █▀▀ █▀█ █▄ █ █ █ █▀▀ █▀█ ▀█▀ █▀▀ █▀█
//!  █▄▄ ▄▄█ ▀▄▀   █▄▄ █▄█ █ ▀█ ▀▄▀ ██▄ █▀▄  █  ██▄ █▀▄
//! ```
//! A tool to convert a CSV file into a new format
//!
//! `csv_converter` is a Rust-based CLI application designed to convert CSV files into a format compatible with
//! [Matrixify](https://matrixify.app/), a powerful import/export app for Shopify stores.
//! This tool streamlines the process of preparing bulk data for Shopify, making it easier to manage large inventories.
//!
//! ```rust
//! use csv_converter::{
//!     Settings,
//!     OutputConfig,
//!     CsvParser,
//!     process,
//!     export,
//! };
//! use std::{io::BufReader, fs::File};
//!
//! let file = BufReader::new(File::open("tests/config.csv").unwrap());
//! let reader = BufReader::new(file);
//! let config_file = CsvParser::new(reader);
//! let output_config = OutputConfig::new(config_file);
//!
//! let mut is_heading = true;
//! let mut output = String::new();
//! let reader = BufReader::new(File::open("tests/input.csv").unwrap());
//! let mut csv_file = CsvParser::new(reader);
//! while let Some(row) = csv_file.next() {
//!     if is_heading {
//!         is_heading = false;
//!         output = format!("{}\n", output_config.heading);
//!     } else {
//!         export(&process::run(&row, &output_config), &mut output);
//!     };
//!     // output is a String with the new content in the format and can now be written to the output file
//! }
//! ```

pub mod cli;
pub mod config;
pub mod csv;
pub mod process;

pub use cli::*;
pub use config::*;
pub use csv::*;
pub use process::*;
