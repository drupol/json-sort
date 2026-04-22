use json_sort::sort_json_file;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_all_fixtures() {
    let fixtures_dir = "tests/fixtures";

    for entry in fs::read_dir(fixtures_dir).expect("Failed to read fixtures directory") {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();

        if path.is_dir() {
            let fixture_name = path.file_name().unwrap().to_string_lossy();

            let input_path = path.join("input.txt");
            let expected_path = path.join("expected.txt");

            let tmp_file = NamedTempFile::new().expect("Failed to create temp file");
            fs::copy(&input_path, tmp_file.path()).expect("Failed to copy fixture");

            sort_json_file(tmp_file.path()).unwrap_or_else(|e| {
                panic!(
                    "Error sorting JSON file in fixture '{}': {}",
                    fixture_name, e
                )
            });

            let result_file =
                fs::read_to_string(tmp_file.path()).expect("Failed to read temp file");

            let expected = fs::read_to_string(&expected_path)
                .unwrap_or_else(|_| panic!("Missing expected.txt in fixture '{}'", fixture_name));

            assert_eq!(
                result_file, expected,
                "Fixture '{}' failed (file content mismatch)",
                fixture_name
            );
        }
    }
}
