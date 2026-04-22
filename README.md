![GitHub stars][github stars]
[![Donate!][donate github]][5]

# JSON Sort

This project is a Rust rewrite of [`json-sort-cli`](https://github.com/tillig/json-sort-cli) with a few differences.

It is designed to make JSON files easier to compare by sorting object keys only. Array order is preserved, so lists are not modified.

## Features

- It is written in [Rust](https://rust-lang.org/) and it is tiny, less than 1.5MB.
- It rewrites the original source file so object keys can be sorted without reformatting the whole document, validates JSON with [`serde_json`](https://crates.io/crates/serde_json).
- It is multi-threaded and can process multiple files in parallel, which may improve performance on large codebases.
- When a file is modified, existing whitespace, indentation, inline layout, and array order are preserved as much as possible.
- It has a `--check` mode for CI usage, which only checks without modifying files.

## Installation

### Via Cargo

You can install the binary with Cargo:

```sh
cargo install json-sort
```

### Via Nixpkgs

Available via the [`json-sort` package][json-sort package], the binary is called `json-sort`.

### Via the source code

Clone the repository and run in the sourcecode folder:

```sh
cargo build --release
```

The binary will be in `target/release/json-sort`.

### Via Nix

You can use the package from this repository with Nix. If you have Nix installed, you can run the tool directly:

```sh
nix run github:drupol/json-sort
```

## Usage

```sh
json-sort [--fix] [--check] [--version] <file_or_dir_or_glob|-> [<other_files_or_dirs_or_globs>]
```

If you pass `-` as the only argument, `json-sort` will read JSON from standard input (stdin) and print the sorted result to standard output (stdout). This is useful for piping and shell usage.

### Main options

- You can pass files, globs (e.g. '\*.json'), or directories as arguments. Directories are searched recursively for `.json` files.
- Resolved files are deduplicated before processing, so overlapping inputs are handled once.
- When multiple files are resolved, they are processed in parallel.
- Inputs that do not resolve to any files are reported as errors.
- A single trailing newline at end of file is preserved and does not make an already sorted file fail checks.
- You can also pass `-` as the only argument to read JSON from stdin and print the sorted result to stdout.
- `--fix`: Update files with fixes instead of just reporting differences.
- `--check`: Only check for unsorted files, never modify them (for CI usage). If both --fix and --check are set, no files will be modified.
- `--version`: Print the CLI version and exit.
- `--fix` exits successfully after applying all fixes, and exits non-zero only when an input or I/O error occurs.

### Examples

- Check sorting of a file:
  ```sh
  json-sort myfile.json
  ```
- Check sorting of all JSON files in a directory (recursively):
  ```sh
  json-sort mydir/
  ```
- Fix all JSON files in the folder (using a glob):
  ```sh
  json-sort --fix '*.json'
  ```
- Fix all JSON files in a directory (recursively):
  ```sh
  json-sort --fix mydir/
  ```
- CI check (fail if any file is not sorted, but do not modify anything):
  ```sh
  json-sort --check '*.json'
  ```
- Pipe JSON content and sort it (output to stdout):
  ```sh
  cat myfile.json | json-sort -
  ```
- Use with curl to pretty-print and sort remote JSON:
  ```sh
  curl -s https://api.example.com/data.json | json-sort -
  ```

## Behaviors

### Comments

Standard JSON does not support comments, but this tool accepts line comments `//` and block comments `/* */` and preserves them when rewriting the file.

When object keys are reordered, comments and surrounding whitespace stay in their original layout positions.

### Normalization

This tool preserves the original literal representation whenever possible:

- **Unicode Escapes**: Escaped strings such as `\u00e9` remain escaped if that is how they appear in the source.
- **Numbers**: Number formatting, including scientific notation, is preserved as written.

### Formatting

This tool does not apply a global pretty-print format. Instead, it rewrites only the object member order and keeps the surrounding formatting intact as much as possible.

Arrays are never reordered, and inline versus multi-line layout is preserved.

## Recommended Tools

For advanced formatting and JSON manipulation, consider these tools:

- [jq](https://jqlang.org/): A flexible command-line JSON processor.
  ```sh
  echo '{"z": 10, "a": 5, "c": 2}' | json-sort - | jq .
  ```
- [Prettier](https://prettier.io/): An opinionated code formatter that supports JSON.
  ```sh
  json-sort --fix myfile.json && prettier --write myfile.json
  ```

[github stars]: https://img.shields.io/github/stars/drupol/json-sort.svg?style=flat-square
[donate github]: https://img.shields.io/badge/Sponsor-Github-brightgreen.svg?style=flat-square
[5]: https://github.com/sponsors/drupol
[json-sort package]: https://search.nixos.org/packages?channel=unstable&from=0&size=50&sort=relevance&type=packages&query=json-sort
