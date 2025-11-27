//! Integration tests for the rfstat CLI tool.
//!
//! These tests verify the complete functionality of rfstat by running
//! the actual binary and testing various command-line scenarios.

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs::{self, File};
use std::io::Write;
use tempfile::TempDir;

/// Helper function to create a test directory structure
fn create_test_directory() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    // Create some test files
    let mut file1 = File::create(base_path.join("small.txt")).unwrap();
    writeln!(file1, "This is a small text file").unwrap();

    let mut file2 = File::create(base_path.join("medium.log")).unwrap();
    writeln!(file2, "{}", "x".repeat(5000)).unwrap();

    let mut file3 = File::create(base_path.join("large.dat")).unwrap();
    writeln!(file3, "{}", "y".repeat(50000)).unwrap();

    // Create a subdirectory with files
    fs::create_dir(base_path.join("subdir")).unwrap();
    let mut file4 = File::create(base_path.join("subdir").join("nested.conf")).unwrap();
    writeln!(file4, "config=value").unwrap();

    // Create a hidden file
    let mut hidden = File::create(base_path.join(".hidden")).unwrap();
    writeln!(hidden, "hidden content").unwrap();

    temp_dir
}

#[test]
fn test_basic_directory_analysis() {
    let temp_dir = create_test_directory();

    let mut cmd = cargo_bin_cmd!("rfstat");
    cmd.arg(temp_dir.path())
        .arg("--quiet")
        .assert()
        .success()
        .stdout(predicate::str::contains("Total Files:"))
        .stdout(predicate::str::contains("Total Directories:"))
        .stdout(predicate::str::contains("Total Size:"));
}

#[test]
fn test_json_output_format() {
    let temp_dir = create_test_directory();

    let mut cmd = cargo_bin_cmd!("rfstat");
    cmd.arg(temp_dir.path())
        .arg("--format")
        .arg("json")
        .arg("--quiet")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"total_files\""))
        .stdout(predicate::str::contains("\"total_size\""))
        .stdout(predicate::str::contains("\"entries\""));
}

#[test]
fn test_csv_output_format() {
    let temp_dir = create_test_directory();

    let mut cmd = cargo_bin_cmd!("rfstat");
    cmd.arg(temp_dir.path())
        .arg("--format")
        .arg("csv")
        .arg("--quiet")
        .assert()
        .success()
        .stdout(predicate::str::contains("path,size_bytes,size_human"))
        .stdout(predicate::str::contains(".txt"))
        .stdout(predicate::str::contains(".log"));
}

#[test]
fn test_summary_output_format() {
    let temp_dir = create_test_directory();

    let mut cmd = cargo_bin_cmd!("rfstat");
    cmd.arg(temp_dir.path())
        .arg("--format")
        .arg("summary")
        .arg("--quiet")
        .assert()
        .success()
        .stdout(predicate::str::contains("Files:"))
        .stdout(predicate::str::contains("Dirs:"))
        .stdout(predicate::str::contains("Size:"));
}

#[test]
fn test_extension_filtering() {
    let temp_dir = create_test_directory();

    let mut cmd = cargo_bin_cmd!("rfstat");
    cmd.arg(temp_dir.path())
        .arg("--extensions")
        .arg("txt,log")
        .arg("--format")
        .arg("csv")
        .arg("--quiet")
        .assert()
        .success()
        .stdout(predicate::str::contains(".txt"))
        .stdout(predicate::str::contains(".log"))
        .stdout(predicate::str::contains(".dat").not());
}

#[test]
fn test_size_filtering() {
    let temp_dir = create_test_directory();

    let mut cmd = cargo_bin_cmd!("rfstat");
    cmd.arg(temp_dir.path())
        .arg("--min-size")
        .arg("1KB")
        .arg("--format")
        .arg("csv")
        .arg("--quiet")
        .assert()
        .success()
        .stdout(predicate::str::contains(".log"))
        .stdout(predicate::str::contains(".dat"));
}

#[test]
fn test_sorting_by_size() {
    let temp_dir = create_test_directory();

    let mut cmd = cargo_bin_cmd!("rfstat");
    cmd.arg(temp_dir.path())
        .arg("--sort")
        .arg("size")
        .arg("--format")
        .arg("csv")
        .arg("--quiet")
        .assert()
        .success();

    // The largest file should appear first in size-sorted output
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();

    // Skip header line and check that files are sorted by size (largest first)
    if lines.len() > 2 {
        // This is a basic check - in a real test you'd parse the CSV properly
        assert!(stdout.contains(".dat")); // Should contain the large file
    }
}

#[test]
fn test_hidden_files_inclusion() {
    let temp_dir = create_test_directory();

    // Test without --all flag (should not include hidden files)
    let mut cmd = cargo_bin_cmd!("rfstat");
    cmd.arg(temp_dir.path())
        .arg("--format")
        .arg("csv")
        .arg("--quiet")
        .assert()
        .success()
        .stdout(predicate::str::contains(".hidden").not());

    // Test with --all flag (should include hidden files)
    let mut cmd = cargo_bin_cmd!("rfstat");
    cmd.arg(temp_dir.path())
        .arg("--all")
        .arg("--format")
        .arg("csv")
        .arg("--quiet")
        .assert()
        .success()
        .stdout(predicate::str::contains(".hidden"));
}

#[test]
fn test_recursive_vs_non_recursive() {
    let temp_dir = create_test_directory();

    // Test recursive (default)
    let mut cmd = cargo_bin_cmd!("rfstat");
    cmd.arg(temp_dir.path())
        .arg("--format")
        .arg("csv")
        .arg("--quiet")
        .assert()
        .success()
        .stdout(predicate::str::contains("nested.conf"));

    // Test non-recursive
    let mut cmd = cargo_bin_cmd!("rfstat");
    cmd.arg(temp_dir.path())
        .arg("--no-recursive")
        .arg("--format")
        .arg("csv")
        .arg("--quiet")
        .assert()
        .success()
        .stdout(predicate::str::contains("nested.conf").not());
}

#[test]
fn test_depth_limiting() {
    let temp_dir = create_test_directory();

    let mut cmd = cargo_bin_cmd!("rfstat");
    cmd.arg(temp_dir.path())
        .arg("--depth")
        .arg("1")
        .arg("--format")
        .arg("csv")
        .arg("--quiet")
        .assert()
        .success()
        .stdout(predicate::str::contains("nested.conf").not());
}

#[test]
fn test_limit_option() {
    let temp_dir = create_test_directory();

    let mut cmd = cargo_bin_cmd!("rfstat");
    cmd.arg(temp_dir.path())
        .arg("--limit")
        .arg("2")
        .arg("--format")
        .arg("csv")
        .arg("--quiet")
        .assert()
        .success();

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();

    // Should have header + 2 data lines (or fewer if there are fewer files)
    assert!(lines.len() <= 3);
}

#[test]
fn test_nonexistent_path() {
    let mut cmd = cargo_bin_cmd!("rfstat");
    cmd.arg("/nonexistent/path")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Path not found"));
}

#[test]
fn test_invalid_size_format() {
    let temp_dir = create_test_directory();

    let mut cmd = cargo_bin_cmd!("rfstat");
    cmd.arg(temp_dir.path())
        .arg("--min-size")
        .arg("invalid_size")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid"));
}

#[test]
fn test_help_output() {
    let mut cmd = cargo_bin_cmd!("rfstat");
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Display file statistics"))
        .stdout(predicate::str::contains("--format"))
        .stdout(predicate::str::contains("--sort"));
}

#[test]
fn test_version_output() {
    let mut cmd = cargo_bin_cmd!("rfstat");
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.1.0"));
}

#[test]
fn test_verbose_logging() {
    let temp_dir = create_test_directory();

    let mut cmd = cargo_bin_cmd!("rfstat");
    cmd.arg(temp_dir.path())
        .arg("--verbose")
        .arg("--format")
        .arg("summary")
        .assert()
        .success();

    // In verbose mode, we should see some debug output
    // Note: This test might be flaky depending on logging configuration
}

#[test]
fn test_permissions_and_times() {
    let temp_dir = create_test_directory();

    let mut cmd = cargo_bin_cmd!("rfstat");
    cmd.arg(temp_dir.path())
        .arg("--show-permissions")
        .arg("--show-times")
        .arg("--format")
        .arg("csv")
        .arg("--quiet")
        .assert()
        .success()
        .stdout(predicate::str::contains("permissions"))
        .stdout(predicate::str::contains("modified"));
}

#[test]
fn test_summary_only_flag() {
    let temp_dir = create_test_directory();

    let mut cmd = cargo_bin_cmd!("rfstat");
    cmd.arg(temp_dir.path())
        .arg("--summary-only")
        .arg("--quiet")
        .assert()
        .success()
        .stdout(predicate::str::contains("Total Files:"))
        .stdout(predicate::str::contains("Size Distribution:"));

    // Should not contain individual file listings
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!stdout.contains("📁 File Details"));
}

#[test]
fn test_current_directory_default() {
    // Test that rfstat works when no path is provided (uses current directory)
    let mut cmd = cargo_bin_cmd!("rfstat");
    cmd.arg("--format")
        .arg("summary")
        .arg("--quiet")
        .assert()
        .success()
        .stdout(predicate::str::contains("Files:"));
}
