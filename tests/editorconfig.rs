use editorconfig_rs::{EditorConfigHandle, ParseError, Version};
use rand::Rng;
use std::{collections::HashMap, fs, os::raw::c_int, path};

const DEFAULT_CONFIG_FILENAME: &str = ".editorconfig";

#[test]
fn new_handle() {
    if let Err(err) = EditorConfigHandle::new() {
        panic!("{}", err);
    }
}

#[test]
fn get_version() {
    let handle = EditorConfigHandle::new().unwrap();
    let expected_version = Version::new(0, 0, 0);
    assert_eq!(handle.get_version(), expected_version);
}

#[test]
fn set_get_version() {
    let mut rng = rand::rng();

    for _ in 1..1000 {
        let handle = EditorConfigHandle::new().unwrap();

        let version = Version {
            major: rng.random_range(0..c_int::MAX),
            minor: rng.random_range(0..c_int::MAX),
            patch: rng.random_range(0..c_int::MAX),
        };

        handle.set_version(version);
        assert_eq!(handle.get_version(), version);
    }
}

#[test]
fn get_config_filename() {
    let handle = EditorConfigHandle::new().unwrap();
    let config_filename = handle.get_config_filename();
    assert!(config_filename.is_none());
}

#[test]
fn set_get_config_filename() {
    let mut handle = EditorConfigHandle::new().unwrap();
    handle.set_config_filename(DEFAULT_CONFIG_FILENAME);

    let config_filename = handle.get_config_filename().unwrap();
    assert_eq!(config_filename, DEFAULT_CONFIG_FILENAME);
}

#[test]
fn parse_config_file_and_get_rules_for_rust_file() {
    // As defined in .editorconfig
    let mut rs_file_rules = HashMap::new();
    rs_file_rules.insert("charset".to_string(), "utf-8".to_string());
    rs_file_rules.insert("end_of_line".to_string(), "lf".to_string());
    rs_file_rules.insert("insert_final_newline".to_string(), "true".to_string());
    rs_file_rules.insert("trim_trailing_whitespace".to_string(), "true".to_string());

    // We use this .rs file for testing, but libeditorconfig requires absolute paths
    let test_file_path = fs::canonicalize(file!()).unwrap();

    let handle = EditorConfigHandle::new().unwrap();
    let err = handle.parse(test_file_path);
    assert!(err.is_none());

    let rules = handle.get_rules();
    assert_eq!(rules.len(), rs_file_rules.len());
    assert_eq!(rules, rs_file_rules);
}

#[test]
fn parse_emoji_path() {
    let emoji_test_path = fs::canonicalize("tests/🦀🚀").unwrap();

    let handle = EditorConfigHandle::new().unwrap();
    let err = handle.parse(emoji_test_path);
    assert!(err.is_none());

    let rule_count = handle.get_rule_count();
    assert_eq!(rule_count, 2);

    let rules = handle.get_rules();
    assert_eq!(rules.len(), 2);
}

#[test]
fn no_parse_get_rules() {
    let handle = EditorConfigHandle::new().unwrap();
    let rules = handle.get_rules();
    assert_eq!(rules.len(), 0);
}

#[test]
fn relative_file_path_error() {
    let handle = EditorConfigHandle::new().unwrap();
    let err = handle.parse(file!()).unwrap();
    assert_eq!(err, ParseError::NotFullPathError);
}

#[test]
fn version_too_new_error() {
    let max_version = Version::new(c_int::MAX, c_int::MAX, c_int::MAX);
    let test_file_path = fs::canonicalize(file!()).unwrap();

    let handle = EditorConfigHandle::new().unwrap();
    handle.set_version(max_version);

    let err = handle.parse(test_file_path).unwrap();
    assert_eq!(err, ParseError::VersionTooNewError);
}

#[test]
fn get_error_message_parse_error() {
    let mut rng = rand::rng();

    // Any error > 0 is a parsing error at that line
    let parse_err_line_num = rng.random_range(1..=c_int::MAX);

    let parse_err = ParseError::LineError(parse_err_line_num);
    let parse_err_msg = editorconfig_rs::get_error_message(parse_err).unwrap();

    // Tight coupling to libeditorconfig's error messages could be improved
    assert_eq!(parse_err_msg, "Failed to parse file.");
}

#[test]
fn get_error_message_relative_path_error() {
    let relative_path_err_msg =
        editorconfig_rs::get_error_message(ParseError::NotFullPathError).unwrap();

    // Tight coupling to libeditorconfig's error messages could be improved
    assert_eq!(
        relative_path_err_msg,
        "Input file must be a full path name."
    );
}

#[test]
fn get_error_message_memory_error() {
    let memory_err_msg = editorconfig_rs::get_error_message(ParseError::MemoryError).unwrap();

    // Tight coupling to libeditorconfig's error messages could be improved
    assert_eq!(memory_err_msg, "Memory error.");
}

#[test]
fn get_error_message_version_error() {
    let version_err_msg =
        editorconfig_rs::get_error_message(ParseError::VersionTooNewError).unwrap();

    // Tight coupling to libeditorconfig's error messages could be improved
    assert_eq!(
        version_err_msg,
        "Required version is greater than the current version."
    );
}

#[test]
fn get_error_file() {
    let mut handle = EditorConfigHandle::new().unwrap();

    let invalid_config_filename = ".editorconfig.invalid";
    let invalid_config_file_path =
        fs::canonicalize(path::Path::new("tests/.editorconfig.invalid")).unwrap();

    // We use this .rs file for testing, but this could be any file as we are
    // only interested in the errors from an invalid config file when parsing it
    let test_file_path = fs::canonicalize(file!()).unwrap();

    // Parse test file with the default and valid config file
    let err = handle.parse(&test_file_path);
    assert!(err.is_none());

    // No error, no error file
    let err_file_path = handle.get_error_file();
    assert!(err_file_path.is_none());

    // Set invalid config filename
    handle.set_config_filename(invalid_config_filename);

    // Parse test file with an invalid config file
    let err = handle.parse(test_file_path).unwrap();
    let ParseError::LineError(err_line_num) = err else {
        panic!("Expected Error::ParseAtLine");
    };
    assert_eq!(err_line_num, 3);

    let err_file_path = handle.get_error_file().unwrap();
    assert_eq!(err_file_path, invalid_config_file_path);
}

#[test]
fn get_rule_count() {
    let handle = EditorConfigHandle::new().unwrap();

    // We use this .rs file for testing, but libeditorconfig requires absolute paths
    let test_file_path = fs::canonicalize(file!()).unwrap();

    let err = handle.parse(test_file_path);
    assert!(err.is_none());

    assert_eq!(handle.get_rule_count(), 4);
}

#[test]
fn lib_get_version() {
    let version = editorconfig_rs::get_version();

    // libeditorconfig 0.12.5 is currently the minimum supported version
    assert!(version.major >= 0);
    assert!(version.minor >= 12);
    assert!(version.patch >= 5);
}

#[test]
fn copy_clone_versions() {
    // Testing the `Clone` and `Copy` traits
    let mut version = Version::new(0, 1, 2);
    let mut version_copy = version;
    let version_clone = version.clone();

    assert_eq!(version, version_copy);
    assert_eq!(version, version_clone);

    // Change the original version
    version.major = 1;
    assert_ne!(version, version_copy);
    assert_ne!(version, version_clone);

    // Change the copy
    version_copy.major = 1;
    assert_eq!(version_copy, version);
    assert_ne!(version_copy, version_clone);
}

#[test]
fn compare_versions() {
    // Testing the `PartialOrd` and `PartialEq` traits
    assert!(Version::new(0, 0, 0) == Version::new(0, 0, 0));
    assert!(Version::new(0, 0, 0) <= Version::new(0, 0, 1));
    assert!(Version::new(0, 0, 1) <= Version::new(0, 1, 0));
    assert!(Version::new(0, 1, 0) <= Version::new(1, 0, 0));
}

#[test]
#[should_panic(expected = "Version numbers cannot be negative")]
fn safe_version() {
    // Testing the "safe" `Version` constructor
    Version::new(-1, -2, -3);
}
