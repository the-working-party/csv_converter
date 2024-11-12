```
█▀▀ █▀▀ █ █   ▀█▀ █▀█   █▀▄▀█ ▄▀█ ▀█▀ █▀█ █ ▀▄▀ █ █▀▀ █▄█
█▄▄ ▄▄█ ▀▄▀    █  █▄█   █ ▀ █ █▀█  █  █▀▄ █ █ █ █ █▀   █
```
A tool to build a matrixify compatible CSV

## Description

`csv2matrixify` is a Rust-based CLI application designed to convert CSV files into a format compatible with
[Matrixify](https://matrixify.app/), a powerful import/export app for Shopify stores.
This tool streamlines the process of preparing bulk data for Shopify, making it easier to manage large inventories.

## Features

- **Easy Conversion**: Transform standard CSV files into Matrixify-compatible CSVs with a single command.
- **Fast Processing**: Leverages Rust's performance to handle *very* large files efficiently.
- **No dependencies**: This app uses no external crates.

## Installation

### Prerequisites

- **Rust**: Ensure you have Rust installed.
You can download it from [rust-lang.org](https://www.rust-lang.org/tools/install).

#### Install via cargo

```sh
cargo install csv2matrixify
```

#### Build from Source

```sh
git clone https://github.com/the-working-party/csv2matrixify.git
cd csv2matrixify
cargo build --release
```

## Usage

```sh
csv2matrixify [OPTIONS]

Options:
  -i <file>, --input <file>
        Specify the input file to process.
  -o <file>, --output <file>
        Specify the output file to write results to.
  -v, -V, --version
        Display the program's version information.
  -h, --help
        Display this help message.
```

Example command:

```sh
csv2matrixify -i input.csv -o output.csv
```

## Contributing

Contributions are welcome.
Please [open an issue](https://github.com/the-working-party/csv2matrixify/issues/new) or
[submit a pull request](https://github.com/the-working-party/csv2matrixify/compare) on the
[GitHub repository](https://github.com/the-working-party/csv2matrixify) to contribute to this project.

## Licensing
Copyleft (c) 2024
Licensed under [MIT](https://raw.githubusercontent.com/the-working-party/csv2matrixify/refs/heads/main/LICENSE?token=GHSAT0AAAAAABO36GVRGUHXFAY4O4AZ6BAQZZSUEGA).
