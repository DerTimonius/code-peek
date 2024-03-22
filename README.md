# Code Peek

_Code Peek_ is a command-line tool written in Rust that helps you gain insights into your codebases. It recursively traverses a directory, counts the lines of code for each file, and provides a summarized output based on your preferences.

## Features

- **Line Count**: Get the line count for each file in the specified directory.
- **Top Files**: Display the top N files with the highest line counts.
- **Grouping**: Group the output by file extension or programming language.
- **Exclusions**: Exclude specific files or patterns from the analysis using globs.
- **Git Integration**: Get information about the number of commits made to each file.

## Installation

_Code Peek_ can be installed directly from the GitHub repository using cargo install (you need to have Rust installed):

```sh copy
cargo install --git https://github.com/DerTimonius/code-peek.git
```

This will download the latest version of Code Peek and install it in your Cargo bin directory (~/.cargo/bin/). Make sure that ~/.cargo/bin is in your system's PATH for easy access to the code-peek command.

## Usage

```sh
code-peek [FLAGS] [OPTIONS] [ARGS]
```

### Flags

- _-a, --all_: Display all available information.
- _-g, --group_: Group the results by file extension or programming language.
- _-t, --git_: Get Git information (number of commits) for each file.

### Options

- _-d, --dir_ <DIR>: Directory to search (defaults to the current working directory).
- _-n, --num_ <NUM>: Number of files to display (defaults to 10).
- _-e, --exclude_ <GLOB>: Globs to exclude files or directories other than those specified in the .gitignore file. Expects a comma-separated list (e.g., '\*.txt,\*.csv').
- _-m, --match_ <GLOB>: Globs to check, expects a comma separated list. E.g. '\*.txt,\*.csv' (Only files that match the pattern will be processed)

## Examples

Display the top 10 files with the highest line counts in the current directory:

```sh copy
code-peek
```

Display the top 20 files in the /path/to/project directory, grouped by file extension:

```sh copy
code-peek -d /path/to/project -n 20 -g
```

Display all available information, excluding .txt and .csv files:

```sh copy
code-peek -a -e '*.txt,*.csv'
```

Display all available information, processing only _Astro_ and _Svelte_ files:

```sh copy
code-peek -a -m '*.astro,*.svelte'
```

## Contributing

Contributions are welcome! Please open an issue or submit a pull request if you have any improvements, bug fixes, or new features to propose.

## License

Code Peek is licensed under the MIT License.
