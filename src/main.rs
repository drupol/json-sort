use anyhow::{Context, Result};
use clap::Parser;
use glob::glob;
use json_sort::{sort_json_file, sort_json_string};
use rayon::prelude::*;
use std::collections::HashSet;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(name = "json-sort")]
#[command(version)]
#[command(
    about = env!("CARGO_PKG_DESCRIPTION")
)]
struct Args {
    /// Update files with fixes instead of just reporting. Defaults to reporting only.
    #[arg(long = "fix")]
    fix: bool,

    /// Only check for unsorted files, never modify them (for CI usage). Overrides --fix.
    #[arg(long = "check")]
    check: bool,

    /// Files, directories, globs, or '-' to read from stdin (must be used alone).
    #[arg(required = true)]
    files: Vec<String>,
}

enum FileStatus {
    Clean,
    Unsorted,
    Fixed,
    Error(String),
}

struct FileResult {
    path: PathBuf,
    status: FileStatus,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.files.len() == 1 && args.files[0] == "-" {
        process_stdin()?;
        return Ok(());
    }

    let (files_to_process, mut had_errors) = collect_files(&args.files)?;
    let mut had_unsorted = false;

    let results: Vec<FileResult> = files_to_process
        .into_par_iter()
        .map(|path| {
            let status: FileStatus = if args.check || !args.fix {
                match check_file(&path) {
                    Ok(true) => FileStatus::Unsorted,
                    Ok(false) => FileStatus::Clean,
                    Err(e) => FileStatus::Error(e.to_string()),
                }
            } else {
                match sort_json_file(&path) {
                    Ok(true) => FileStatus::Fixed,
                    Ok(false) => FileStatus::Clean,
                    Err(e) => FileStatus::Error(e.to_string()),
                }
            };
            FileResult { path, status }
        })
        .collect();

    // The current tests expect output to be printed in order, but Rayon processing is out of order.
    // However, we can sort the results by path to maintain some determinism or just print them.
    // The previous implementation used an index to keep original order. Let's do that if needed.
    // But sorting by path is also fine and maybe better for users.
    let mut results = results;
    results.sort_by(|a, b| a.path.cmp(&b.path));

    for result in results {
        match result.status {
            FileStatus::Clean => {}
            FileStatus::Unsorted => {
                had_unsorted = true;
                eprintln!("{:?} is not properly sorted.", result.path);
            }
            FileStatus::Fixed => {
                eprintln!("File updated: {:?}", result.path);
            }
            FileStatus::Error(message) => {
                had_errors = true;
                eprintln!("Error processing {:?}: {}", result.path, message);
            }
        }
    }

    if had_errors || had_unsorted {
        std::process::exit(1);
    }

    Ok(())
}

fn process_stdin() -> Result<()> {
    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .context("Failed to read from stdin")?;

    let sorted = sort_json_string(&buffer)?;
    println!("{}", sorted);
    Ok(())
}

fn check_file(path: &Path) -> Result<bool> {
    let original =
        fs::read_to_string(path).with_context(|| format!("Failed to read file {:?}", path))?;

    let sorted = sort_json_string(&original)
        .with_context(|| format!("Failed to sort JSON in {:?}", path))?;

    Ok(original != sorted)
}

fn collect_files(patterns: &[String]) -> Result<(Vec<PathBuf>, bool)> {
    let mut files_to_process = Vec::new();
    let mut seen_paths = HashSet::new();
    let mut had_errors = false;

    for pattern in patterns {
        let path = Path::new(pattern);
        if path.is_dir() {
            for entry in walkdir::WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file()
                    && entry.path().extension().is_some_and(|ext| ext == "json")
                {
                    add_unique_path(
                        &mut files_to_process,
                        &mut seen_paths,
                        entry.path().to_path_buf(),
                    )?;
                }
            }
        } else if pattern.contains('*') || pattern.contains('?') || pattern.contains('[') {
            let mut matched = false;
            for entry in glob(pattern).context("Failed to read glob pattern")? {
                let p = entry.context("Error expanding glob pattern")?;
                if p.is_file() {
                    matched = true;
                    add_unique_path(&mut files_to_process, &mut seen_paths, p)?;
                }
            }
            if !matched {
                eprintln!("No files matched input: {}", pattern);
                had_errors = true;
            }
        } else {
            if path.exists() {
                add_unique_path(&mut files_to_process, &mut seen_paths, path.to_path_buf())?;
            } else {
                eprintln!("No files matched input: {}", pattern);
                had_errors = true;
            }
        }
    }

    Ok((files_to_process, had_errors))
}

fn add_unique_path(
    files: &mut Vec<PathBuf>,
    seen: &mut HashSet<PathBuf>,
    path: PathBuf,
) -> Result<()> {
    let canonical = fs::canonicalize(&path)
        .with_context(|| format!("Failed to canonicalize path {:?}", path))?;

    if seen.insert(canonical) {
        files.push(path);
    }
    Ok(())
}
