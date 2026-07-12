//! Process-level acceptance tests for the S003 configuration-backed vertical flow.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use serde_json::Value;

const SECRET_STDERR: &str = "FAKE_STDERR_SECRET_SENTINEL";
const SECRET_LOG: &str = "FAKE_LOGMESSAGE_SECRET_SENTINEL";
const SECRET_ARG: &str = "FAKE_CONFIG_ARG_SECRET_SENTINEL";

struct Fixture {
    root: PathBuf,
    config: PathBuf,
    workspace: PathBuf,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn unique_root(label: &str) -> PathBuf {
    static FIXTURE_SEQ: AtomicU64 = AtomicU64::new(0);
    std::env::temp_dir().join(format!(
        "mango-lsp-vf-{}-{}-{}",
        label,
        std::process::id(),
        FIXTURE_SEQ.fetch_add(1, Ordering::Relaxed)
    ))
}

fn fake_server() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mango-lsp-fake-server"))
}

fn mango_lsp() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mango-lsp"))
}

fn write_config(path: &Path, command: &Path, args: &[&str]) {
    let args_toml = if args.is_empty() {
        String::new()
    } else {
        let rendered = args
            .iter()
            .map(|arg| format!("\"{}\"", arg.replace('\\', "\\\\").replace('"', "\\\"")))
            .collect::<Vec<_>>()
            .join(", ");
        format!("args = [{rendered}]\n")
    };
    let command_text = command.display().to_string().replace('\\', "/");
    fs::write(
        path,
        format!(
            "schema_version = 1\n\n[server]\nid = \"fixture\"\ncommand = \"{command_text}\"\n{args_toml}"
        ),
    )
    .expect("write config");
}

fn make_fixture(label: &str, mode: &str, extra_args: &[&str]) -> Fixture {
    let root = unique_root(label);
    let workspace = root.join("workspace");
    fs::create_dir_all(&workspace).expect("workspace");
    let config = root.join("config.toml");
    let mut args = vec![mode];
    args.extend_from_slice(extra_args);
    write_config(&config, &fake_server(), &args);
    Fixture {
        root,
        config,
        workspace,
    }
}

fn run_workspace_symbols(fixture: &Fixture, query: &str) -> Output {
    Command::new(mango_lsp())
        .args([
            "workspace-symbols",
            "--config",
            fixture.config.to_str().unwrap(),
            "--workspace",
            fixture.workspace.to_str().unwrap(),
            "--query",
            query,
        ])
        .env("MANGO_LSP_TEST_SECRET_ENV", "FAKE_ENV_SECRET_SENTINEL")
        .output()
        .expect("mango-lsp should launch")
}

fn parse_envelope(stdout: &[u8]) -> Value {
    let text = String::from_utf8(stdout.to_vec()).expect("stdout utf-8");
    assert!(
        text.ends_with('\n'),
        "stdout must end with newline: {text:?}"
    );
    assert_eq!(
        text.lines().count(),
        1,
        "stdout must be exactly one line: {text:?}"
    );
    serde_json::from_str(text.trim_end()).expect("stdout envelope json")
}

fn parse_events(stderr: &[u8]) -> Vec<Value> {
    let text = String::from_utf8(stderr.to_vec()).expect("stderr utf-8");
    assert!(
        !text.contains("panicked"),
        "stderr must not contain panic output: {text}"
    );
    text.lines()
        .map(|line| {
            serde_json::from_str(line).unwrap_or_else(|error| {
                panic!("stderr line is not JSON ({error}): {line}");
            })
        })
        .collect()
}

fn assert_no_secrets(output: &Output) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    for secret in [
        SECRET_STDERR,
        SECRET_LOG,
        SECRET_ARG,
        "FAKE_ENV_SECRET_SENTINEL",
        "SECRET_QUERY_SHOULD_NOT_LEAK",
    ] {
        assert!(
            !stdout.contains(secret),
            "stdout leaked secret {secret}: {stdout}"
        );
        assert!(
            !stderr.contains(secret),
            "stderr leaked secret {secret}: {stderr}"
        );
    }
}

#[test]
fn vertical_flow_help_documents_required_options() {
    let output = Command::new(mango_lsp())
        .args(["workspace-symbols", "--help"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--config"));
    assert!(stdout.contains("--workspace"));
    assert!(stdout.contains("--query"));
}

#[test]
fn vertical_flow_success_emits_envelope_and_events() {
    let fixture = make_fixture("ok", "workspace-symbols", &[SECRET_ARG]);
    let output = run_workspace_symbols(&fixture, "Widget");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let envelope = parse_envelope(&output.stdout);
    assert_eq!(envelope["schema_version"], 1);
    assert_eq!(envelope["operation"], "workspace_symbols");
    assert_eq!(envelope["status"], "ok");
    assert_eq!(envelope["server"], "fixture");
    assert_eq!(envelope["result"]["symbols"][0]["name"], "Widget");
    assert_eq!(envelope["result"]["symbols"][0]["kind"], "class");
    assert!(envelope["error"].is_null());

    let events = parse_events(&output.stderr);
    let names: Vec<&str> = events
        .iter()
        .map(|event| event["event"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"operation_started"));
    assert!(names.contains(&"child_started"));
    assert!(names.contains(&"downstream_notification"));
    assert!(names.contains(&"child_stopped"));
    assert!(names.contains(&"operation_succeeded"));
    assert_no_secrets(&output);
}

#[test]
fn vertical_flow_redacts_sentinels_on_success() {
    let fixture = make_fixture("redact", "workspace-symbols", &[SECRET_ARG]);
    let output = run_workspace_symbols(&fixture, "SECRET_QUERY_SHOULD_NOT_LEAK");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_no_secrets(&output);
    let events = parse_events(&output.stderr);
    let stopped = events
        .iter()
        .find(|event| event["event"] == "child_stopped")
        .expect("child_stopped");
    assert!(stopped["observed_bytes"].as_u64().unwrap() > 0);
}

#[test]
fn vertical_flow_rejects_invalid_query() {
    let fixture = make_fixture("query", "workspace-symbols", &[]);
    let output = run_workspace_symbols(&fixture, "");
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    let envelope = parse_envelope(&output.stdout);
    assert_eq!(envelope["status"], "error");
    assert_eq!(envelope["error"]["kind"], "query");
    assert_eq!(envelope["error"]["cleanup"], "not_required");
    parse_events(&output.stderr);
}

#[test]
fn vertical_flow_rejects_invalid_config() {
    let root = unique_root("bad-config");
    fs::create_dir_all(&root).unwrap();
    let workspace = root.join("workspace");
    fs::create_dir_all(&workspace).unwrap();
    let config = root.join("config.toml");
    fs::write(
        &config,
        r#"
schema_version = 1
[server]
id = "Bad ID"
command = "/no/such"
"#,
    )
    .unwrap();
    let _guard = Fixture {
        root,
        config: config.clone(),
        workspace: workspace.clone(),
    };
    let output = Command::new(mango_lsp())
        .args([
            "workspace-symbols",
            "--config",
            config.to_str().unwrap(),
            "--workspace",
            workspace.to_str().unwrap(),
            "--query",
            "Widget",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    let envelope = parse_envelope(&output.stdout);
    assert_eq!(envelope["error"]["kind"], "configuration");
}

#[test]
fn vertical_flow_toml_parse_error_redacts_config_contents() {
    let root = unique_root("toml-err");
    fs::create_dir_all(&root).unwrap();
    let workspace = root.join("workspace");
    fs::create_dir_all(&workspace).unwrap();
    let config = root.join("config.toml");
    // Unterminated array on the line carrying the sentinel argument: the
    // parse error must not echo configuration contents into either stream.
    fs::write(
        &config,
        format!(
            "schema_version = 1\n[server]\nid = \"fixture\"\ncommand = \"/usr/bin/true\"\nargs = [\"{SECRET_ARG}\"\n"
        ),
    )
    .unwrap();
    let _guard = Fixture {
        root,
        config: config.clone(),
        workspace: workspace.clone(),
    };
    let output = Command::new(mango_lsp())
        .args([
            "workspace-symbols",
            "--config",
            config.to_str().unwrap(),
            "--workspace",
            workspace.to_str().unwrap(),
            "--query",
            "Widget",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    let envelope = parse_envelope(&output.stdout);
    assert_eq!(envelope["error"]["kind"], "configuration");
    assert_no_secrets(&output);
}

#[test]
fn vertical_flow_rejects_missing_workspace() {
    let fixture = make_fixture("missing-ws", "workspace-symbols", &[]);
    let missing = fixture.root.join("does-not-exist");
    let output = Command::new(mango_lsp())
        .args([
            "workspace-symbols",
            "--config",
            fixture.config.to_str().unwrap(),
            "--workspace",
            missing.to_str().unwrap(),
            "--query",
            "Widget",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    let envelope = parse_envelope(&output.stdout);
    assert_eq!(envelope["error"]["kind"], "workspace");
}

#[test]
fn vertical_flow_spawn_failure() {
    let root = unique_root("spawn");
    fs::create_dir_all(root.join("workspace")).unwrap();
    let config = root.join("config.toml");
    write_config(
        &config,
        Path::new("/definitely/missing/mango-lsp-server"),
        &[],
    );
    let fixture = Fixture {
        root: root.clone(),
        config,
        workspace: root.join("workspace"),
    };
    let output = run_workspace_symbols(&fixture, "Widget");
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let envelope = parse_envelope(&output.stdout);
    assert_eq!(envelope["error"]["kind"], "spawn");
}

#[test]
fn vertical_flow_unsupported_capability() {
    let fixture = make_fixture("no-cap", "no-symbol-capability", &[]);
    let output = run_workspace_symbols(&fixture, "Widget");
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let envelope = parse_envelope(&output.stdout);
    assert_eq!(envelope["error"]["kind"], "unsupported_capability");
    assert_eq!(envelope["error"]["cleanup"], "completed");
}

#[test]
fn vertical_flow_protocol_malformed_symbols() {
    let fixture = make_fixture("malformed", "malformed-symbols", &[]);
    let output = run_workspace_symbols(&fixture, "Widget");
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let envelope = parse_envelope(&output.stdout);
    assert_eq!(envelope["error"]["kind"], "protocol");
}

#[test]
fn vertical_flow_downstream_symbol_error() {
    let fixture = make_fixture("sym-err", "symbol-error", &[]);
    let output = run_workspace_symbols(&fixture, "Widget");
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let envelope = parse_envelope(&output.stdout);
    assert_eq!(envelope["error"]["kind"], "downstream");
}

#[test]
fn vertical_flow_timeout_stall_symbol() {
    let fixture = make_fixture("stall", "stall-symbol", &[]);
    // Override via env is not supported; default 5s timeout is fine for CI.
    let started = Instant::now();
    let output = run_workspace_symbols(&fixture, "Widget");
    let elapsed = started.elapsed();
    assert!(
        elapsed < Duration::from_secs(20),
        "timeout path too slow: {elapsed:?}"
    );
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let envelope = parse_envelope(&output.stdout);
    assert_eq!(envelope["error"]["kind"], "timeout");
}

#[test]
fn vertical_flow_workspace_with_spaces_and_non_ascii() {
    let root = unique_root("uri");
    let workspace = root.join("my workspace").join("café");
    fs::create_dir_all(&workspace).unwrap();
    let config = root.join("config.toml");
    write_config(&config, &fake_server(), &["workspace-symbols"]);
    let fixture = Fixture {
        root,
        config,
        workspace,
    };
    let output = run_workspace_symbols(&fixture, "Widget");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    parse_envelope(&output.stdout);
    parse_events(&output.stderr);
}

#[cfg(windows)]
#[test]
fn vertical_flow_windows_drive_workspace_uri() {
    let fixture = make_fixture("win-uri", "workspace-symbols", &[]);
    let output = run_workspace_symbols(&fixture, "Widget");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    // Success implies URI construction did not fail for the drive path.
    parse_envelope(&output.stdout);
}
