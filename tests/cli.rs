use std::process::{Command, Output};

fn run_mango_lsp(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mango-lsp"))
        .args(arguments)
        .output()
        .expect("mango-lsp should launch")
}

#[test]
fn help_identifies_the_program() {
    let output = run_mango_lsp(&["--help"]);
    assert!(output.status.success(), "--help failed: {output:?}");

    let stdout = String::from_utf8(output.stdout).expect("help output should be UTF-8");
    assert!(!stdout.trim().is_empty(), "help output should not be empty");
    assert!(
        stdout.contains("Usage:"),
        "help should contain usage text: {stdout}"
    );
    assert!(
        stdout.contains("mango-lsp"),
        "help should identify the program: {stdout}"
    );
}

#[test]
fn version_matches_package_metadata() {
    let output = run_mango_lsp(&["--version"]);
    assert!(output.status.success(), "--version failed: {output:?}");

    let stdout = String::from_utf8(output.stdout).expect("version output should be UTF-8");
    assert_eq!(
        stdout.trim(),
        format!("mango-lsp {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn unknown_option_reports_a_useful_error() {
    let output = run_mango_lsp(&["--definitely-unknown"]);
    let stderr = String::from_utf8(output.stderr).expect("error output should be UTF-8");

    assert!(!output.status.success(), "unknown option should fail");
    assert!(
        stderr.contains("error:"),
        "stderr should identify an error: {stderr}"
    );
    assert!(
        stderr.contains("--definitely-unknown"),
        "stderr should identify the unknown option: {stderr}"
    );
    assert!(
        !stderr.contains("panicked"),
        "unknown option should not panic: {stderr}"
    );
}
