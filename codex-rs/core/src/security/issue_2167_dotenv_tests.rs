use super::*;
use pretty_assertions::assert_eq;

#[test]
fn issue_2167_dotenv_shell_read_routes_are_credentials() {
    for command in [
        "cat .env",
        "/bin/cat /work/project/.env",
        "sudo -n chroot / /bin/cat /work/project/.env",
        "sed 's/=.*/=REDACTED/' .env",
        "head .env.local",
        "cat .envrc",
        "cat .ENV",
        "cat .en[v]",
        "cat .env.*",
        "cat < .env",
        "p=.env; cat \"$p\"",
        "cd project && cat .env",
        "python3 -c \"print(open('.env').read())\"",
        "node -e \"console.log(require('fs').readFileSync('.env','utf8'))\"",
    ] {
        assert_eq!(
            script(command),
            Some(ProtectedActionKind::Credentials),
            "{command}"
        );
    }
}

#[cfg(windows)]
#[test]
fn issue_2167_dotenv_windows_read_routes_are_credentials() {
    for command in [
        "Get-Content .env",
        "Get-Content -LiteralPath C:\\work\\project\\.env",
    ] {
        assert_eq!(
            kind(&["powershell", "-NoProfile", "-Command", command]),
            Some(ProtectedActionKind::Credentials),
            "{command}"
        );
    }
    assert_eq!(
        kind(&["cmd", "/c", "type .env"]),
        Some(ProtectedActionKind::Credentials)
    );
}

#[test]
fn issue_2167_dotenv_templates_and_ordinary_files_stay_unprotected() {
    for path in [
        ".env.example",
        ".env.sample",
        ".env.template",
        "notes.env",
        "README.md",
    ] {
        assert_eq!(script(&format!("cat {path}")), None, "{path}");
    }
}

#[test]
fn issue_2167_dotenv_mcp_file_reads_are_credentials() {
    use crate::security::protected_surface::McpCall;
    use crate::security::protected_surface::classify_mcp;
    for path in [
        ".env",
        "/work/project/.env",
        "./project/.env.local",
        ".envrc",
        #[cfg(windows)]
        r"C:\work\project\.env",
    ] {
        let arguments = serde_json::json!({"path": path});
        assert_eq!(
            classify_mcp(
                &McpCall {
                    server: "filesystem",
                    tool: "read_file",
                    title: None,
                    acts_outside: false,
                    arguments: Some(&arguments),
                    cwd: &abs("/work"),
                },
                &fixture(HOME),
            ),
            Some(ProtectedActionKind::Credentials),
            "{path}"
        );
    }
}

#[cfg(unix)]
#[test]
fn issue_2167_dotenv_symlink_read_is_credentials() {
    let workspace = tempfile::tempdir().expect("workspace");
    std::fs::write(workspace.path().join(".env"), "SYNTHETIC=fixture").expect("fixture");
    std::os::unix::fs::symlink(".env", workspace.path().join("notes")).expect("link");
    assert_eq!(
        classify_fixture(&shell_in(
            workspace.path().to_str().expect("path"),
            &["cat", "notes"]
        )),
        Some(ProtectedActionKind::Credentials)
    );
}
