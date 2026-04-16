use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const CLI_PATH: &str = env!("CARGO_BIN_EXE_json-sort");

fn unique_temp_path(suffix: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "json-sort-{suffix}-{}-{timestamp}.json",
        std::process::id()
    ))
}

#[test]
fn test_check_flag_reports_unsorted() {
    let file_path = unique_temp_path("check");
    let unsorted = "{\"b\":1,\"a\":2}";
    fs::write(&file_path, unsorted).unwrap();

    let output = Command::new(CLI_PATH)
        .arg("--check")
        .arg(&file_path)
        .output()
        .expect("failed to execute process");

    // Should exit with code 1 and print the file as not properly sorted
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("is not properly sorted"));

    // File should not be modified
    let after = fs::read_to_string(&file_path).unwrap();
    assert_eq!(after, unsorted);

    fs::remove_file(file_path).unwrap();
}

#[test]
fn test_fix_updates_file_and_exits_successfully() {
    let file_path = unique_temp_path("fix");
    fs::write(&file_path, "{\"b\":1,\"a\":2}").unwrap();

    let output = Command::new(CLI_PATH)
        .arg("--fix")
        .arg(&file_path)
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let after = fs::read_to_string(&file_path).unwrap();
    assert_eq!(after, "{\"a\":2,\"b\":1}");

    fs::remove_file(file_path).unwrap();
}

#[test]
fn test_missing_input_exits_with_error() {
    let file_path = unique_temp_path("missing");

    let output = Command::new(CLI_PATH)
        .arg(&file_path)
        .output()
        .expect("failed to execute process");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No files matched input"));
}

#[test]
fn test_duplicate_inputs_are_processed_once() {
    let file_path = unique_temp_path("duplicate");
    fs::write(&file_path, "{\"b\":1,\"a\":2}").unwrap();

    let output = Command::new(CLI_PATH)
        .arg("--check")
        .arg(&file_path)
        .arg(&file_path)
        .output()
        .expect("failed to execute process");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(stderr.matches("is not properly sorted").count(), 1);

    fs::remove_file(file_path).unwrap();
}

#[test]
fn test_check_accepts_sorted_file_with_trailing_newline() {
    let file_path = unique_temp_path("trailing-newline-check");
    fs::write(&file_path, "{\n  \"a\": 2,\n  \"b\": 1\n}\n").unwrap();

    let output = Command::new(CLI_PATH)
        .arg("--check")
        .arg(&file_path)
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.is_empty());

    fs::remove_file(file_path).unwrap();
}

#[test]
fn test_fix_preserves_trailing_newline_at_eof() {
    let file_path = unique_temp_path("trailing-newline-fix");
    fs::write(&file_path, "{\"b\":1,\"a\":2}\n").unwrap();

    let output = Command::new(CLI_PATH)
        .arg("--fix")
        .arg(&file_path)
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let after = fs::read_to_string(&file_path).unwrap();
    assert_eq!(after, "{\"a\":2,\"b\":1}\n");

    fs::remove_file(file_path).unwrap();
}

#[test]
fn test_version_prints_package_version() {
    let output = Command::new(CLI_PATH)
        .arg("--version")
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        stdout.trim(),
        concat!("json-sort ", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn test_help_includes_package_description() {
    let output = Command::new(CLI_PATH)
        .arg("--help")
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(env!("CARGO_PKG_DESCRIPTION")));
}
