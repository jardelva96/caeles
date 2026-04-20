use assert_cmd::Command;
use predicates::str::contains;
use serde_json::Value;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

const CAPSULE_ID: &str = "com.caeles.test.demo";
const CAPSULE_NAME: &str = "Demo Capsule";
const CAPSULE_VERSION: &str = "0.1.0";

fn run_caeles(workdir: &Path) -> Command {
    let mut cmd =
        Command::cargo_bin("caeles").expect("caeles binary should be available for tests");
    cmd.current_dir(workdir);
    cmd
}

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent directory should be created");
    }
    fs::write(path, content).expect("file should be written");
}

fn demo_wat_module() -> &'static str {
    r#"(module
  (import "caeles" "host_log" (func $host_log (param i32 i32)))
  (import "caeles" "host_notify" (func $host_notify (param i32 i32)))
  (import "caeles" "host_http_get" (func $host_http_get (param i32 i32) (result i32)))
  (memory (export "memory") 1)
  (data (i32.const 0) "integration-log")
  (data (i32.const 64) "integration-notify")
  (data (i32.const 128) "https://example.com")
  (func (export "caeles_main")
    i32.const 0
    i32.const 15
    call $host_log
    i32.const 64
    i32.const 18
    call $host_notify
    i32.const 128
    i32.const 19
    call $host_http_get
    drop
  )
)
"#
}

fn write_demo_capsule_wasm(path: &Path) {
    let bytes = wat::parse_str(demo_wat_module()).expect("WAT should compile to valid wasm");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("wasm parent dir should be created");
    }
    fs::write(path, bytes).expect("wasm file should be written");
}

fn write_demo_registry_fixture(workdir: &Path, lifecycle_kind: &str) {
    let manifest_path = workdir.join("capsules/demo/manifest.json");
    let wasm_path = workdir.join("capsules/demo/demo.wasm");
    let registry_path = workdir.join("capsules/registry.json");

    write_demo_capsule_wasm(&wasm_path);

    let manifest = serde_json::json!({
        "id": CAPSULE_ID,
        "name": CAPSULE_NAME,
        "version": CAPSULE_VERSION,
        "entry": "demo.wasm",
        "permissions": {
            "notifications": true,
            "network": false
        },
        "lifecycle": {
            "kind": lifecycle_kind
        }
    });
    write_file(
        &manifest_path,
        &serde_json::to_string_pretty(&manifest).expect("manifest json should serialize"),
    );

    let registry = serde_json::json!([
        {
            "id": CAPSULE_ID,
            "name": CAPSULE_NAME,
            "manifest": "demo/manifest.json"
        }
    ]);
    write_file(
        &registry_path,
        &serde_json::to_string_pretty(&registry).expect("registry json should serialize"),
    );
}

fn write_build_capsule_fixture(workdir: &Path) {
    let cargo_toml = r#"[package]
name = "build-capsule"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]
"#;

    let lib_rs = r#"#[no_mangle]
pub extern "C" fn caeles_main() {}
"#;

    write_file(
        &workdir.join("capsules/build-capsule/Cargo.toml"),
        cargo_toml,
    );
    write_file(&workdir.join("capsules/build-capsule/src/lib.rs"), lib_rs);
}

fn extract_run_id(stdout: &str) -> String {
    stdout
        .split_whitespace()
        .find(|token| token.starts_with("run-"))
        .expect("run id should be present in command output")
        .to_owned()
}

#[test]
fn cli_definition_of_done_happy_path() {
    let temp = TempDir::new().expect("temp directory should be created");
    write_demo_registry_fixture(temp.path(), "on_demand");

    let list_stdout = run_caeles(temp.path())
        .args(["list", "--registry", "capsules/registry.json", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let listed: Value = serde_json::from_slice(&list_stdout).expect("list output should be json");
    assert_eq!(listed.as_array().map(Vec::len), Some(1));

    let run_stdout = run_caeles(temp.path())
        .args([
            "run",
            "--capsule-id",
            CAPSULE_ID,
            "--registry",
            "capsules/registry.json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let run_stdout = String::from_utf8(run_stdout).expect("run output should be utf-8");
    assert!(
        run_stdout.contains("[capsule-network BLOCKED]"),
        "run output should show runtime network enforcement"
    );
    let run_id = extract_run_id(&run_stdout);

    let inspect_run_stdout = run_caeles(temp.path())
        .args(["inspect-run", &run_id, "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let inspect_run: Value =
        serde_json::from_slice(&inspect_run_stdout).expect("inspect-run output should be json");
    assert_eq!(inspect_run["run_id"].as_str(), Some(run_id.as_str()));
    assert_eq!(inspect_run["capsule_id"].as_str(), Some(CAPSULE_ID));
    assert_eq!(inspect_run["status"].as_str(), Some("exited"));

    let logs_stdout = run_caeles(temp.path())
        .args(["logs", &run_id, "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let logs: Value = serde_json::from_slice(&logs_stdout).expect("logs output should be json");
    let logs = logs.as_array().expect("logs output should be array");
    assert!(logs.iter().any(|line| {
        line.as_str()
            .map(|value| value.contains("starting capsule"))
            .unwrap_or(false)
    }));
    assert!(logs.iter().any(|line| {
        line.as_str()
            .map(|value| value.contains("runtime_exit: success"))
            .unwrap_or(false)
    }));

    let inspect_stdout = run_caeles(temp.path())
        .args([
            "inspect",
            CAPSULE_ID,
            "--registry",
            "capsules/registry.json",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let inspect: Value =
        serde_json::from_slice(&inspect_stdout).expect("inspect output should be json");
    assert_eq!(inspect["id"].as_str(), Some(CAPSULE_ID));
    assert_eq!(inspect["manifest_exists"].as_bool(), Some(true));

    run_caeles(temp.path())
        .args([
            "package",
            "--capsule-id",
            CAPSULE_ID,
            "--registry",
            "capsules/registry.json",
            "--output-dir",
            ".caeles/packages",
        ])
        .assert()
        .success();

    let package_root = temp.path().join(format!(
        ".caeles/packages/{}/{}/",
        CAPSULE_ID, CAPSULE_VERSION
    ));
    assert!(package_root.join("manifest.json").exists());
    assert!(package_root.join("capsule.wasm").exists());
    assert!(package_root.join("package.json").exists());

    run_caeles(temp.path())
        .args([
            "pull",
            CAPSULE_ID,
            "--registry",
            "capsules/registry.json",
            "--output-dir",
            ".caeles/pulled",
        ])
        .assert()
        .success();

    let pulled_root = temp.path().join(format!(
        ".caeles/pulled/{}/{}/",
        CAPSULE_ID, CAPSULE_VERSION
    ));
    assert!(pulled_root.join("manifest.json").exists());
    assert!(pulled_root.join("capsule.wasm").exists());

    let images_stdout = run_caeles(temp.path())
        .args([
            "images",
            "--packages-dir",
            ".caeles/packages",
            "--pulled-dir",
            ".caeles/pulled",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let images: Value =
        serde_json::from_slice(&images_stdout).expect("images output should be valid json");
    assert_eq!(images.as_array().map(Vec::len), Some(2));
}

#[test]
fn cli_build_subcommand_builds_capsule_for_v0_target() {
    let temp = TempDir::new().expect("temp directory should be created");
    write_build_capsule_fixture(temp.path());

    run_caeles(temp.path())
        .args(["build", "capsules/build-capsule"])
        .assert()
        .success();

    assert!(temp
        .path()
        .join("target/wasm32-unknown-unknown/debug/build_capsule.wasm")
        .exists());
}

#[test]
fn cli_run_fails_for_invalid_manifest_lifecycle() {
    let temp = TempDir::new().expect("temp directory should be created");
    write_demo_registry_fixture(temp.path(), "boot");

    run_caeles(temp.path())
        .args(["run", "--manifest", "capsules/demo/manifest.json"])
        .assert()
        .failure()
        .stderr(contains("Manifest invalido"))
        .stderr(contains("on_demand"));
}
