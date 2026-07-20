![GitHub stars][GitHub stars]
[![NPM Version][NPM Version]][NPM package]
[![Crates.io Version][Crates.io Version]][json-sort crates]
[![Crates.io License][Crates.io License]][json-sort crates]
[![Try online][try online badge]][try online link]
[![Donate!][Donate!]][sponsor link]

# JSON Sort

This project is a Rust rewrite of [`json-sort-cli`](https://github.com/tillig/json-sort-cli) with a few differences.

It is designed to make JSON files easier to compare by sorting object keys only. Array order is preserved, so lists are not modified.

Thanks to Rust's WebAssembly support, the same sorting logic is also available as a Node.js package.

## Features

- It is written in [Rust](https://rust-lang.org/) and it is tiny, less than 1.5MB.
- It rewrites the original source file so object keys can be sorted without reformatting the whole document, while validating JSON string literals with [`serde_json`](https://crates.io/crates/serde_json).
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

Clone the repository and run in the source code folder:

```sh
cargo build --release
```

The binary will be in `target/release/json-sort`.

### Via Node.js

A Node.js package generated from the Rust WebAssembly build is available as [`json-sort-rs`][json-sort npm].

```sh
npm install json-sort-rs
```

```js
import { sort_json_string } from "json-sort-rs";

const sorted = sort_json_string('{"z":1,"a":2}');

console.log(sorted);
```

### Browser demo with WebAssembly

The library can also run in a browser through WebAssembly:

```sh
just serve
```

Open <http://localhost:8080> and use the page to sort JSON object keys locally
in the browser.

Try the online demo on GitHub Pages; the link is available in the badge at the top of this
file.

### Via Nix

You can use the package from this repository with Nix. If you have Nix installed, you can run the tool directly:

```sh
nix run github:drupol/json-sort
```

### Treefmt module

This repository also exposes a reusable `treefmtModule` as `treefmtModules.default`, so you can add the
`json-sort` formatter to any `treefmt` configuration without duplicating the setup.

```nix
  treefmt = {
    imports = [
      inputs.json-sort.treefmtModules.default
    ];
    projectRootFile = "flake.nix";
    programs = {
      json-sort.enable = true;
    };
  };
```

Make sure to add the `json-sort` flake input:

```nix
  inputs = {
    # ... 8< ...
    json-sort.url = "github:drupol/json-sort";
    # ... >8 ...
  };
```

## Usage

### Command Line

```sh
json-sort [--fix] [--check] [--version] <file_or_dir_or_glob|-> [<other_files_or_dirs_or_globs>]
```

If you pass `-` as the only argument, `json-sort` will read JSON from standard input (stdin) and print the sorted result to standard output (stdout). This is useful for piping and shell usage.

### VSCode Extension

You can integrate `json-sort` in VS Code using the [Custom Local Formatters extension](https://marketplace.visualstudio.com/items?itemName=jkillian.custom-local-formatters).

Add the following to `.vscode/settings.json`:

```json
{
  "customLocalFormatters.formatters": [
    {
      "command": "json-sort --fix -",
      "languages": ["json"]
    }
  ]
}
```

This runs `json-sort` as the formatter for JSON files.

How to use it:

1. Install `json-sort`.
2. Edit `.vscode/settings.json` and add the configuration above.
3. Open a JSON file.
4. Press `Shift+Ctrl+P`, run `Format Document With...`, then select `Custom Local Formatters`.
5. Profit.

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

Standard JSON does not support comments, but this tool accepts line comments `//` and block comments `/* */` and preserves them when rewriting the file. In practice, this makes the accepted input closer to JSONC than strict JSON.

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

- [jaq]: A `jq` clone focussed on correctness, speed, and simplicity
  ```sh
  echo '{"z": 10, "a": 5, "c": 2}' | json-sort - | jaq .
  ```
- [Prettier]: An opinionated code formatter that supports JSON
  ```sh
  json-sort --fix myfile.json && prettier --write myfile.json
  ```

[GitHub stars]: https://img.shields.io/github/stars/drupol/json-sort.svg?style=flat-square
[Donate!]: https://img.shields.io/badge/Sponsor-Github-brightgreen.svg?style=flat-square
[sponsor link]: https://github.com/sponsors/drupol
[`json-sort` package]: https://search.nixos.org/packages?channel=unstable&from=0&size=50&sort=relevance&type=packages&query=json-sort
[MiniJinja]: https://docs.rs/minijinja/
[Crates.io License]: https://img.shields.io/crates/l/json-sort?style=flat-square
[Crates.io Version]: https://img.shields.io/crates/v/json-sort?style=flat-square
[json-sort package]: https://search.nixos.org/packages?channel=unstable&from=0&size=50&sort=relevance&type=packages&query=json-sort
[json-sort crates]: https://crates.io/crates/json-sort
[json-sort npm]: https://www.npmjs.com/package/json-sort-rs
[jaq]: https://github.com/01mf02/jaq
[Prettier]: https://prettier.io/
[try online badge]: https://img.shields.io/badge/Online_Demo-green?style=flat-square
[try online link]: https://drupol.github.io/json-sort/
[NPM Version]: https://img.shields.io/npm/v/json-sort-rs?style=flat-square
[NPM package]: https://www.npmjs.com/package/json-sort-rs
