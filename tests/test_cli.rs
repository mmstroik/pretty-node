use assert_cmd::Command;
use predicates::prelude::*;

/// Test CLI command execution and argument parsing
#[cfg(test)]
mod cli_tests {
    use super::*;

    #[test]
    fn test_help_flag() {
        let mut cmd = Command::cargo_bin("pretty-node").unwrap();
        cmd.arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("Usage:"));
    }

    #[test]
    fn test_no_args_shows_help() {
        let mut cmd = Command::cargo_bin("pretty-node").unwrap();
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("Usage:"));
    }

    #[test]
    fn test_tree_command_basic() {
        let mut cmd = Command::cargo_bin("pretty-node").unwrap();
        cmd.args(&["tree", "lodash"])
            .assert()
            .success()
            .stdout(predicate::str::contains("📦"));
    }

    #[test]
    fn test_tree_command_with_depth() {
        let mut cmd = Command::cargo_bin("pretty-node").unwrap();
        cmd.args(&["tree", "lodash", "--depth", "1"])
            .assert()
            .success()
            .stdout(predicate::str::contains("📦"));
    }

    #[test]
    fn test_tree_command_quiet() {
        let mut cmd = Command::cargo_bin("pretty-node").unwrap();
        cmd.args(&["tree", "lodash", "--quiet"])
            .assert()
            .success()
            .stdout(predicate::str::contains("📦"));
    }

    #[test]
    fn test_tree_command_json_output() {
        let mut cmd = Command::cargo_bin("pretty-node").unwrap();
        cmd.args(&["tree", "lodash", "--output", "json"])
            .assert()
            .success()
            .stdout(predicate::str::contains("\"name\":"))
            .stdout(predicate::str::contains("\"version\":"));
    }

    #[test]
    fn test_sig_command_basic() {
        let mut cmd = Command::cargo_bin("pretty-node").unwrap();
        cmd.args(&["sig", "lodash:isArray"])
            .assert()
            .success()
            .stdout(predicate::str::contains("📎"));
    }

    #[test]
    fn test_sig_command_json_output() {
        let mut cmd = Command::cargo_bin("pretty-node").unwrap();
        cmd.args(&["sig", "lodash:isArray", "--output", "json"])
            .assert()
            .success()
            .stdout(predicate::str::contains("\"name\":"))
            .stdout(predicate::str::contains("\"parameters\":"));
    }

    #[test]
    fn test_sig_nonexistent_function() {
        let mut cmd = Command::cargo_bin("pretty-node").unwrap();
        cmd.args(&["sig", "nonexistent:function"])
            .assert()
            .success()
            .stdout(predicate::str::contains("signature not available"));
    }

    #[test]
    fn test_tree_with_colon_syntax_error() {
        let mut cmd = Command::cargo_bin("pretty-node").unwrap();
        cmd.args(&["tree", "package:symbol"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("Invalid module path"));
    }

    #[test]
    fn test_nonexistent_package() {
        let mut cmd = Command::cargo_bin("pretty-node").unwrap();
        cmd.args(&["tree", "this-package-definitely-does-not-exist-12345"])
            .timeout(std::time::Duration::from_secs(30))
            .assert()
            .success(); // Should complete without crashing
    }

    #[test]
    fn test_version_flag() {
        let mut cmd = Command::cargo_bin("pretty-node").unwrap();
        cmd.arg("--version")
            .assert()
            .success()
            .stdout(predicate::str::contains("pretty-node"));
    }

    #[test]
    fn test_invalid_command() {
        let mut cmd = Command::cargo_bin("pretty-node").unwrap();
        cmd.args(&["invalid-command", "some-package"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("error"));
    }

    #[test]
    fn test_missing_package_argument() {
        let mut cmd = Command::cargo_bin("pretty-node").unwrap();
        cmd.arg("tree")
            .assert()
            .failure()
            .stderr(predicate::str::contains("required"));
    }

    #[test]
    fn test_auto_download_functionality() {
        // Test with a small, stable package
        let mut cmd = Command::cargo_bin("pretty-node").unwrap();
        cmd.args(&["tree", "ms", "--quiet", "--depth", "1"])
            .timeout(std::time::Duration::from_secs(60))
            .assert()
            .success()
            .stdout(predicate::str::contains("📦"));
    }
}