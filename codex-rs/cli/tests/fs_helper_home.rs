//! #847: private file helpers must not run the interactive Corbanu-home preflight.
//! Synthetic profiles only. The real macOS sandbox still rejects protected paths.
#![cfg(target_os = "macos")]

use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::process::Output;
use std::process::Stdio;

use anyhow::Result;
use pretty_assertions::assert_eq;
use serde_json::Value;
use serde_json::json;
use tempfile::TempDir;

struct Fixture {
    root: TempDir,
    canonical_root: std::path::PathBuf,
}

impl Fixture {
    fn new() -> Result<Self> {
        let root = TempDir::new()?;
        std::fs::create_dir_all(root.path().join("account/.corbanu"))?;
        std::fs::create_dir_all(root.path().join("project"))?;
        std::fs::create_dir_all(root.path().join("denied"))?;
        std::fs::write(
            root.path().join("account/.corbanu/private.txt"),
            "synthetic-profile-canary",
        )?;
        std::fs::write(
            root.path().join("project/notes.txt"),
            "synthetic project notes",
        )?;
        std::fs::write(
            root.path().join("denied/private.txt"),
            "synthetic-outside-canary",
        )?;
        let canonical_root = root.path().canonicalize()?;
        Ok(Self {
            root,
            canonical_root,
        })
    }

    fn path(&self, relative: &str) -> std::path::PathBuf {
        self.canonical_root.join(relative)
    }

    fn policy(&self) -> Result<String> {
        // Never widen product policy. This extra test sandbox denies the toy
        // profile and outside path, all networking and native credential services.
        Ok(format!(
            "(version 1)(allow default)(deny network*)\n\
             (deny mach-lookup (global-name \"com.apple.SecurityServer\") \
             (global-name \"com.apple.securityd.xpc\") \
             (global-name \"com.apple.securityd.systemkeychain\"))\n\
             (deny file-read* file-write* (subpath {}) (subpath {}))",
            serde_json::to_string(&self.path("account/.corbanu"))?,
            serde_json::to_string(&self.path("denied"))?,
        ))
    }

    fn run(&self, args: &[&str], input: &[u8]) -> Result<Output> {
        let mut child = Command::new("/usr/bin/sandbox-exec")
            .args(["-p", &self.policy()?])
            .arg(codex_utils_cargo_bin::cargo_bin("corbanu")?)
            .args(args)
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("HOME", self.path("account"))
            .env("TMPDIR", self.root.path())
            .env("CORBANU_TEST_NO_NATIVE_KEYRING", "1")
            .env("CORBANU_TEST_ACCOUNT_HOME", self.path("account"))
            .current_dir(self.path("project"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        // Profile aliases are intentionally absent, like the actual helper's
        // environment allowlist. HOME is synthetic to avoid operator state.
        child
            .stdin
            .take()
            .ok_or_else(|| anyhow::anyhow!("stdin was not piped"))?
            .write_all(input)?;
        Ok(child.wait_with_output()?)
    }

    fn helper(&self, request: Value) -> Result<Value> {
        let output = self.run(
            &[codex_exec_server::CODEX_FS_HELPER_ARG1],
            &serde_json::to_vec(&request)?,
        )?;
        assert!(
            output.status.success(),
            "private file helper failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(serde_json::from_slice(&output.stdout)?)
    }
}

fn uri(path: &Path) -> Result<String> {
    url::Url::from_file_path(path)
        .map(|url| url.to_string())
        .map_err(|()| anyhow::anyhow!("fixture path is not absolute: {}", path.display()))
}

#[test]
fn allowed_project_read_works_with_a_denied_default_profile() -> Result<()> {
    let fixture = Fixture::new()?;
    let response = fixture.helper(json!({
        "operation": "fs/readFile",
        "params": {"path": uri(&fixture.path("project/notes.txt"))?}
    }))?;
    assert_eq!(
        response,
        json!({
            "status": "ok", "payload": {
                "operation": "fs/readFile", "response": {
                    "dataBase64": "c3ludGhldGljIHByb2plY3Qgbm90ZXM="
                }
            }
        })
    );
    Ok(())
}

#[test]
fn allowed_project_write_and_repeated_read_work() -> Result<()> {
    let fixture = Fixture::new()?;
    for (data, expected) in [
        ("Zmlyc3QgZWRpdA==", "first edit"),
        ("c2Vjb25kIGVkaXQ=", "second edit"),
    ] {
        let response = fixture.helper(json!({
            "operation": "fs/writeFile", "params": {
                "path": uri(&fixture.path("project/notes.txt"))?, "dataBase64": data
            }
        }))?;
        assert_eq!(
            response,
            json!({"status":"ok","payload":{"operation":"fs/writeFile","response":{}}})
        );
        assert_eq!(
            std::fs::read_to_string(fixture.path("project/notes.txt"))?,
            expected
        );
        let response = fixture.helper(json!({
            "operation": "fs/readFile", "params": {"path": uri(&fixture.path("project/notes.txt"))?}
        }))?;
        assert_eq!(response["payload"]["response"]["dataBase64"], data);
    }
    Ok(())
}

#[test]
fn protected_profile_and_other_denied_paths_remain_denied() -> Result<()> {
    let fixture = Fixture::new()?;
    for relative in ["account/.corbanu/private.txt", "denied/private.txt"] {
        let target = fixture.path(relative);
        let original = std::fs::read(&target)?;
        for request in [
            json!({"operation":"fs/readFile","params":{"path":uri(&target)?}}),
            json!({"operation":"fs/writeFile","params":{"path":uri(&target)?,"dataBase64":"bm90IGFsbG93ZWQ="}}),
        ] {
            let response = fixture.helper(request)?;
            assert_eq!(response["status"], "error");
            assert!(!response.to_string().contains("canary"));
            assert!(
                response
                    .get("payload")
                    .and_then(|p| p.get("response"))
                    .is_none()
            );
        }
        assert_eq!(std::fs::read(&target)?, original);
    }
    Ok(())
}

#[test]
fn malformed_private_request_reaches_the_helper_protocol_error() -> Result<()> {
    let fixture = Fixture::new()?;
    let output = fixture.run(&[codex_exec_server::CODEX_FS_HELPER_ARG1], b"not-json")?;
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("fs sandbox helper failed:"));
    assert!(!stderr.contains("macOS is blocking access"));
    Ok(())
}

#[test]
fn ordinary_corbanu_invocation_keeps_the_home_privacy_preflight() -> Result<()> {
    let fixture = Fixture::new()?;
    let output = fixture.run(&["--version"], b"")?;
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("macOS is blocking access to the Corbanu home"));
    assert!(
        stderr.contains(
            &fixture
                .path("account/.corbanu")
                .to_string_lossy()
                .to_string()
        )
    );
    Ok(())
}

#[test]
fn native_sandbox_controls_work_before_any_product_startup() -> Result<()> {
    let fixture = Fixture::new()?;
    let invoke = |program: &str, args: &[&std::ffi::OsStr]| -> Result<Output> {
        Ok(Command::new("/usr/bin/sandbox-exec")
            .args(["-p", &fixture.policy()?])
            .arg(program)
            .args(args)
            .env_clear()
            .env("HOME", fixture.path("account"))
            .env("PATH", "/usr/bin:/bin")
            .current_dir(fixture.path("project"))
            .output()?)
    };
    let project = fixture.path("project/notes.txt");
    let positive = invoke("/bin/cat", &[project.as_os_str()])?;
    assert!(positive.status.success());
    assert_eq!(positive.stdout, b"synthetic project notes");
    for relative in ["account/.corbanu/private.txt", "denied/private.txt"] {
        let target = fixture.path(relative);
        let original = std::fs::read(&target)?;
        let denied = invoke("/bin/cat", &[target.as_os_str()])?;
        assert!(!denied.status.success());
        assert!(denied.stdout.is_empty());
        let denied = invoke(
            "/bin/sh",
            &[
                std::ffi::OsStr::new("-c"),
                std::ffi::OsStr::new("printf forbidden > \"$1\""),
                std::ffi::OsStr::new("fixture"),
                target.as_os_str(),
            ],
        )?;
        assert!(!denied.status.success());
        assert_eq!(std::fs::read(&target)?, original);
    }
    Ok(())
}
