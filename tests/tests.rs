use json_sort::sort_json_string;
use std::fs;

#[test]
fn test_all_fixtures() {
    let fixtures_dir = "tests/fixtures";

    // Ensure the directory exists
    if !std::path::Path::new(fixtures_dir).exists() {
        panic!("Fixtures directory {} not found", fixtures_dir);
    }

    for entry in fs::read_dir(fixtures_dir).expect("Failed to read fixtures directory") {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();

        if path.is_dir() {
            let fixture_name = path.file_name().unwrap().to_string_lossy();

            let input_path = path.join("input.txt");
            let expected_path = path.join("expected.txt");

            let input = fs::read_to_string(&input_path)
                .unwrap_or_else(|_| panic!("Missing input.txt in fixture '{}'", fixture_name));
            let expected = fs::read_to_string(&expected_path)
                .unwrap_or_else(|_| panic!("Missing expected.txt in fixture '{}'", fixture_name));

            let result = sort_json_string(&input).unwrap_or_else(|e| {
                panic!("Error sorting JSON in fixture '{}': {}", fixture_name, e)
            });

            assert_eq!(result, expected, "Fixture '{}' failed", fixture_name);
        }
    }
}
