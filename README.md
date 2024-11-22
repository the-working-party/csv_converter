```
 █▀▀ █▀▀ █ █   █▀▀ █▀█ █▄ █ █ █ █▀▀ █▀█ ▀█▀ █▀▀ █▀█
 █▄▄ ▄▄█ ▀▄▀   █▄▄ █▄█ █ ▀█ ▀▄▀ ██▄ █▀▄  █  ██▄ █▀▄
```
A tool to convert a CSV file into a new format

<p align="center">
	<a href="https://crates.io/crates/csv_converter"><img src="https://img.shields.io/crates/v/csv_converter.svg" alt="crates badge"></a>
	<a href="https://docs.rs/csv_converter/"><img src="https://docs.rs/csv_converter/badge.svg" alt="crates docs"></a>
	<a href="https://github.com/the-working-party/csv_converter/actions/workflows/testing.yml"><img src="https://github.com/the-working-party/csv_converter/actions/workflows/testing.yml/badge.svg" alt="build status"></a>
</p>

## Description

`csv_converter` is a Rust-based CLI application designed to convert CSV files into any format you want driven by a powerful config.
At [The Working Party](https://theworkingparty.com.au/) we use this tool to streamline the process of preparing bulk product data for Shopify imports,
making it easier to import massive inventories.

## Features

- **Easy Conversion**: Transform standard CSV files into a CSVs layout you need with a config file easily written with any spreadsheet processor.
- **Fast Processing**: Leverages Rust's performance to handle *very* large files efficiently.
- **No dependencies**: This app uses no external crates.

## How does it work?

Imagine you scrape a website with your favorite scraper and now have this huge spreadsheet with a lot of data.
![The input CSV](assets/input.png)
<details>
<summary>View the raw CSV</summary>

```csv
URL,name,image1,image2,image3,SKU,description,data1,data2,variant1,variant2
https://myshop.tld/product/berta2-green-holster,Berta2,https://cdn.myshop.tld/img1.jpg,https://cdn.myshop.tld/img2.jpg,https://cdn.myshop.tld/img3.jpg,berta2,Berta2 is the new and improved berta,,,black,green
https://myshop.tld/product/susan-organic,Susan,https://cdn.myshop.tld/img1.jpg,https://cdn.myshop.tld/img2.jpg,https://cdn.myshop.tld/img3.jpg,susan,Buy Susan,,,organic,toxic
```
</details>

These spreadsheet can be very large and contain many cells that you may not even need.
Others need to be reshuffled or split into it's own line etc.

A good spreadsheet for the above data could be this sheet:
![The output CSV](assets/output.png)
<details>
<summary>View the raw CSV</summary>

```csv
Handle,Command,Name,Description,Variant ID,Variant Command,Option1 Name,Option1 Value
berta2,NEW,Berta2,Berta2 is the new and improved berta,,MERGE,Material,black
berta2,MERGE,,,,MERGE,Material,green
susan,NEW,Susan,Buy Susan,,MERGE,Material,organic
berta2,MERGE,,,,MERGE,Material,toxic
```
</details>

You have to split off each line into two and make sure you select the right items with the right headlines.

With `csv_converter` you can do this by creating a config spreadsheet like this:
![The config CSV](assets/config.png)
<details>
<summary>View the raw CSV</summary>

```csv
Handle,Command,Name,Description,Variant ID,Variant Command,Option1 Name,Option1 Value
<cell6>,NEW,<cell2>,<cell7>,,MERGE,Material,<cell10>
<cell6>,MERGE,,,,MERGE,Material,<cell11>
```
</details>

The first line of the config is the heading you like.
No changes will be made to it.

All lines after are free for you to allocate.
You reference cells by using the `<cell[x]>` token.
The reference is pointing to a single line from your import.
Each line from you input CSV file will be processed via this config.

<details>
<summary>Show more</summary>

```
   ┌────────────────────────────────────────────────────────────┐
   │                         Input.csv                          │
   ├───────┬───────┬───────────┬─────────┬──────────────┬───────┤
   │Heading│Heading│  Heading  │ Heading │   Heading    │Heading│
   ├───────┼───────┼───────────┼─────────┼──────────────┼───────┤
   │<cell1>│<cell2>│  <cell3>  │ <cell4> │   <cell5>    │<cell6>│
   ├───────┼───────┼───────────┼─────────┼──────────────┼───────┤
   │  ...  │  ...  │    ...    │   ...   │     ...      │  ...  │
   ├───────┼───────┼───────────┼─────────┼──────────────┼───────┤
   │  ...  │  ...  │    ...    │   ...   │     ...      │  ...  │
   ├───────┼───────┼───────────┼─────────┼──────────────┼───────┤
   │  ...  │  ...  │    ...    │   ...   │     ...      │  ...  │
   └───────┴───────┴───────────┴─────────┴──────────────┴───────┘
                                 │
                                 │
                                 ▼
┌───────────────────────────────────────────────────────────────────┐
│                            Config.csv                             │
├───────┬───────────┬─────────┬──────────────┬───────┬──────────────┤
│Heading│  Heading  │ Heading │   Heading    │Heading│   Heading    │
├───────┼───────────┼─────────┼──────────────┼───────┼──────────────┤
│<cell6>│  <cell3>  │  MERGE  │   <cell5>    │<cell1>│ https://...  │
├───────┼───────────┼─────────┼──────────────┼───────┼──────────────┤
│<cell6>│  <cell4>  │   NEW   │              │<cell2>│ https://...  │
└───────┴───────────┴─────────┴──────────────┴───────┴──────────────┘
                                 │
                                 │
                                 ▼
┌───────────────────────────────────────────────────────────────────┐
│                            Output.csv                             │
├───────┬───────────┬─────────┬──────────────┬───────┬──────────────┤
│Heading│  Heading  │ Heading │   Heading    │Heading│   Heading    │
├───────┼───────────┼─────────┼──────────────┼───────┼──────────────┤
│  ...  │    ...    │  MERGE  │     ...      │  ...  │ https://...  │
├───────┼───────────┼─────────┼──────────────┼───────┼──────────────┤
│  ...  │    ...    │   NEW   │              │  ...  │ https://...  │
├───────┼───────────┼─────────┼──────────────┼───────┼──────────────┤
│  ...  │    ...    │  MERGE  │     ...      │  ...  │ https://...  │
├───────┼───────────┼─────────┼──────────────┼───────┼──────────────┤
│  ...  │    ...    │   NEW   │              │  ...  │ https://...  │
├───────┼───────────┼─────────┼──────────────┼───────┼──────────────┤
│  ...  │    ...    │  MERGE  │     ...      │  ...  │ https://...  │
├───────┼───────────┼─────────┼──────────────┼───────┼──────────────┤
│  ...  │    ...    │   NEW   │              │  ...  │ https://...  │
├───────┼───────────┼─────────┼──────────────┼───────┼──────────────┤
│  ...  │    ...    │  MERGE  │     ...      │  ...  │ https://...  │
├───────┼───────────┼─────────┼──────────────┼───────┼──────────────┤
│  ...  │    ...    │   NEW   │              │  ...  │ https://...  │
├───────┼───────────┼─────────┼──────────────┼───────┼──────────────┤
│  ...  │    ...    │  MERGE  │     ...      │  ...  │ https://...  │
├───────┼───────────┼─────────┼──────────────┼───────┼──────────────┤
│  ...  │    ...    │   NEW   │              │  ...  │ https://...  │
└───────┴───────────┴─────────┴──────────────┴───────┴──────────────┘
```

In this example we're splitting a single input line into two resulting in double the lines in our output file.
</details>


## Config reference

The config file includes logic and filters that will make it easier for you to generate smarter outputs.

### Filters

Filters allow you to make changes to the content of a cell.

Syntax: `<cell[n] FILTER|'argument'|[number]>`

_(💡  You can combine filters simply by adding them: `<cell1 TRIM APPEND|'!!!' UPPER_CASE>` which will give us this: `HELLO WORLD!!!`)_

For the below documentation we assume `<cell1>` has the value `  Hello World  `

#### `UPPER_CASE`
Convert the contents of a cell into upper case.
- `<cell1 UPPER_CASE>` => `  HELLO WORLD  `

#### `LOWER_CASE`
Convert the contents of a cell into lower case.
- `<cell1 LOWER_CASE>` => `  hello world  `

#### `LENGTH`
Convert the contents of a cell into the number of characters it contains.
- `<cell1 LENGTH>` => `15`

#### `TRIM`
Removes whitespace from both ends of the cell.
- `<cell1 TRIM>` => `Hello World`

#### `TRIM_START`
Removes whitespace from the start of the cell.
- `<cell1 TRIM_START>` => `Hello World  `

#### `TRIM_END`
Removes whitespace from the end of the cell.
- `<cell1 TRIM_END>` => `  Hello World`

#### `REPLACE|' '|'-'`
Replaces something of the cell with something else.
- `<cell1 REPLACE|'World'|'Everyone'>` => `  Hello Everyone  `

#### `APPEND|'-end'`
Adds something to the end of the cell.
- `<cell1 APPEND|'!!!'>` => `  Hello World  !!!`

#### `PREPEND|'pre-'`
Adds something to the start of the cell.
- `<cell1 PREPEND|':)'>` => `:)  Hello World  `

#### `SPLIT|','|1`
Splits the cell every time it finds the string you pass in and allows you to select which of the resulting bits you want to show.
- `<cell1 SPLIT|'o'|1>` => ` W`

#### `SUB_STRING|10|5`
Returns only a part of the cell by you defining the start and optionally the end.
If the end is not given the rest of the cell will be returned.
- `<cell1 SUB_STRING|1>` => `Hello World  `
- `<cell1 SUB_STRING|1|5>` => `  Hello World  `

## Conditions

Conditions allow you to add logic to a cell.

Syntax: `:IF <cell1> [condition] ('then-item') [ELSE ('else-item')]`

- The `ELSE` clause is optional
- A `then-item` can be a String or a cell: `:IF <cell1> [condition] ('then-item')` or `:IF <cell1> [condition] (<cell2>)`
- All cells inside a condition support all filters

_(💡  If any of your conditions evaluate to `SKIP_THIS_LINE` then the entire line won't be exported in the output)_

#### `IS_EMPTY`
Checks if the cell is empty.
- `:IF <cell1> IS_EMPTY (<cell2>)`

#### `IS_NOT_EMPTY`
Checks if the cell is not empty.
- `:IF <cell1> IS_NOT_EMPTY (<cell2>)`

#### `IS_NUMERIC`
Checks if the cell is a number.
- `:IF <cell1> IS_NUMERIC (<cell2>)`

#### `STARTS_WITH|'beginning'`
Checks if the cell starts with a given string.
- `:IF <cell1> STARTS_WITH|'beginning' (<cell2>)`

#### `ENDS_WITH|'end'`
Checks if the cell ends with a given string.
- `:IF <cell1> ENDS_WITH|'end' (<cell2>)`

#### `CONTAINS|'happiness'`
Checks if the cell contains a given string.
- `:IF <cell1> CONTAINS|'happiness' (<cell2>)`

#### `== 'this item`
Checks if the cell is equal to a given string.
- `:IF <cell1> == 'Same?' (<cell2>)`

#### `!= 'this item`
Checks if the cell is not equal to a given string.
- `:IF <cell1> != 'Not the Same?' (<cell2>)`

#### `> 42`
Checks if the cell is greater than a given number.
- `:IF <cell1> > 42 (<cell2>)`

#### `< 42`
Checks if the cell is less than a given number.
- `:IF <cell1> <> 42 (<cell2>)`

#### `% 2 = 0`
Checks if the cell, when divided by a given number, leaves a remainder equal to a given value.
- `:IF <cell1> % 2 = 0 (<cell2>)`

## CLI Usage

```sh
csv_converter [OPTIONS]

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
        Display this help message.
```

Example command:

```sh
csv_converter -i input.csv -o output.csv -c config.csv
```

## Installation

### Prerequisites

- **Rust**: Ensure you have Rust installed.
You can download it from [rust-lang.org](https://www.rust-lang.org/tools/install).

#### Install via cargo

```sh
cargo install csv_converter
```

#### Build from Source

```sh
git clone https://github.com/the-working-party/csv_converter.git
cd csv_converter
cargo build --release
# Now run the app via "cargo run --release" instead of "csv_converter" or locate the binary in your target folder
```

## Contributing

Contributions are welcome.
Please [open an issue](https://github.com/the-working-party/csv_converter/issues/new) or
[submit a pull request](https://github.com/the-working-party/csv_converter/compare) on the
[GitHub repository](https://github.com/the-working-party/csv_converter) to contribute to this project.

## Licensing
Copyleft (c) 2024
Licensed under [MIT](https://raw.githubusercontent.com/the-working-party/csv_converter/refs/heads/main/LICENSE?token=GHSAT0AAAAAABO36GVRGUHXFAY4O4AZ6BAQZZSUEGA).
