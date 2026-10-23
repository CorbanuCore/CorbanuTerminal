//! #477: `corbanu exec` stops the curated plugin sync before it exits. Its
//! `git` (and the helpers `git` starts) no longer outlive exec and keep
//! writing a staged clone into the home.
#![cfg(unix)]

use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;

use codex_protocol::user_input::MAX_USER_INPUT_TEXT_CHARS;
use core_test_support::responses::ev_assistant_message;
use core_test_support::responses::ev_completed;
use core_test_support::responses::ev_response_created;
use core_test_support::responses::sse;
use core_test_support::responses::sse_response;
use core_test_support::responses::start_mock_server;
use core_test_support::test_codex_exec::test_codex_exec;
use wiremock::Mock;
use wiremock::Request;
use wiremock::Respond;
use wiremock::ResponseTemplate;
use wiremock::matchers::method;
use wiremock::matchers::path_regex;

/// Answers the turn only once the fake `git fetch` runs, so exec always
/// shuts down with the sync in flight.
struct AfterFetchStarts {
    marker: PathBuf,
    body: String,
}

impl Respond for AfterFetchStarts {
    fn respond(&self, _request: &Request) -> ResponseTemplate {
        wait_until(Duration::from_secs(20), || self.marker.exists());
        sse_response(self.body.clone())
    }
}

fn wait_until(limit: Duration, done: impl Fn() -> bool) -> bool {
    let deadline = Instant::now() + limit;
    while !done() {
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    true
}

fn read_pid(path: &Path) -> libc::pid_t {
    std::fs::read_to_string(path)
        .expect("pid file")
        .trim()
        .parse()
        .expect("pid")
}

fn alive(pid: libc::pid_t) -> bool {
    // SAFETY: signal 0 only checks that the process exists.
    unsafe { libc::kill(pid, 0) == 0 }
}

fn clone_dirs(home: &Path) -> Vec<PathBuf> {
    std::fs::read_dir(home.join(".tmp"))
        .map(|entries| {
            entries
                .flatten()
                .map(|entry| entry.path())
                .filter(|path| {
                    path.file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| name.starts_with("plugins-clone-"))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// A fake `git` on `PATH`: a slow `git fetch` that keeps writing the staged
/// clone's shallow.lock, with a helper in its process group. Bounded so a
/// failure leaks nothing for long.
struct FakeGit {
    bin: tempfile::TempDir,
    git_pid: PathBuf,
    helper_pid: PathBuf,
}

impl FakeGit {
    fn new() -> Self {
        let bin = tempfile::tempdir().expect("fake git dir");
        let git_pid = bin.path().join("git.pid");
        let helper_pid = bin.path().join("helper.pid");
        let script = format!(
            r#"#!/bin/sh
if [ "$1" = "ls-remote" ]; then
  printf '%s\tHEAD\n' 0123456789abcdef0123456789abcdef01234567
  exit 0
fi
if [ "$1" = "-C" ] && [ "$3" = "init" ]; then
  mkdir -p "$2/.git"
  exit 0
fi
if [ "$1" = "-C" ] && [ "$3" = "fetch" ]; then
  sleep 30 &
  echo $! > "{helper}"
  echo $$ > "{git}.tmp" && mv "{git}.tmp" "{git}"
  i=0
  while [ $i -lt 600 ]; do date >> "$2/.git/shallow.lock"; sleep 0.05; i=$((i+1)); done
  exit 1
fi
exit 1
"#,
            helper = helper_pid.display(),
            git = git_pid.display(),
        );
        write_executable(&bin.path().join("git"), &script);
        Self {
            bin,
            git_pid,
            helper_pid,
        }
    }

    fn path_env(&self) -> String {
        format!(
            "{}:{}",
            self.bin.path().display(),
            std::env::var("PATH").unwrap_or_default()
        )
    }

    /// Asserts the sync ran and that nothing it started outlived exec.
    fn assert_stopped(&self, home: &Path) {
        assert!(
            self.git_pid.exists(),
            "the plugin sync never ran the fake git"
        );
        let pids = [read_pid(&self.git_pid), read_pid(&self.helper_pid)];
        // Killed before exec exits; allow only for the kernel to finish reaping.
        let stopped = wait_until(Duration::from_secs(2), || {
            !pids.iter().any(|pid| alive(*pid))
        });
        for pid in pids {
            if alive(pid) {
                // SAFETY: cleanup of the fake git this test started.
                unsafe { libc::kill(pid, libc::SIGKILL) };
            }
        }
        assert!(stopped, "plugin sync git outlived codex-exec");
        assert_eq!(clone_dirs(home), Vec::<PathBuf>::new());
    }
}

fn write_executable(path: &Path, script: &str) {
    std::fs::write(path, script).expect("write script");
    std::fs::set_permissions(path, std::os::unix::fs::PermissionsExt::from_mode(0o755))
        .expect("chmod script");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn exec_stops_the_plugin_sync_git_before_it_exits() {
    let test = test_codex_exec();
    let git = FakeGit::new();

    let server = start_mock_server().await;
    Mock::given(method("POST"))
        .and(path_regex(".*/responses$"))
        .respond_with(AfterFetchStarts {
            marker: git.git_pid.clone(),
            body: sse(vec![
                ev_response_created("resp-1"),
                ev_assistant_message("msg-1", "hi"),
                ev_completed("resp-1"),
            ]),
        })
        .mount(&server)
        .await;

    let output = test
        .cmd_with_server(&server)
        .env("PATH", git.path_env())
        .arg("--skip-git-repo-check")
        .arg("say hi")
        .output()
        .expect("run codex-exec");

    assert!(output.status.success(), "exec failed: {output:?}");
    git.assert_stopped(test.home_path());
}

/// #494: an early error exit after the in-process client started (here
/// `turn/start` rejects an oversized prompt) still shuts the client down, so
/// the sync's `git` does not outlive exec either.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn exec_stops_the_plugin_sync_git_when_turn_start_fails() {
    let test = test_codex_exec();
    let git = FakeGit::new();
    // A required MCP server holds `thread/start` until the fake `git fetch`
    // runs, so `turn/start` always fails with the sync in flight. It answers
    // every request with an empty result.
    let mcp = git.bin.path().join("gate-mcp");
    write_executable(
        &mcp,
        &format!(
            r#"#!/bin/sh
i=0
while [ ! -e "{git}" ] && [ $i -lt 400 ]; do sleep 0.05; i=$((i+1)); done
[ -e "{git}" ] || exit 1
while IFS= read -r line; do
  id=$(printf '%s' "$line" | sed -n 's/.*"id":\([0-9][0-9]*\).*/\1/p')
  [ -n "$id" ] || continue
  case "$line" in
    *'"method":"initialize"'*)
      version=$(printf '%s' "$line" | sed -n 's/.*"protocolVersion":"\([^"]*\)".*/\1/p')
      result='{{"protocolVersion":"'"$version"'","capabilities":{{"tools":{{}}}},"serverInfo":{{"name":"gate","version":"0"}}}}' ;;
    *'"method":"tools/list"'*) result='{{"tools":[]}}' ;;
    *) result='{{}}' ;;
  esac
  printf '{{"jsonrpc":"2.0","id":%s,"result":%s}}\n' "$id" "$result"
done
"#,
            git = git.git_pid.display(),
        ),
    );
    std::fs::write(
        test.home_path().join("config.toml"),
        format!(
            "[mcp_servers.gate]\ncommand = {}\nrequired = true\nstartup_timeout_sec = 30\n",
            // A JSON string is a valid TOML basic string.
            serde_json::to_string(&mcp.display().to_string()).expect("serialize path")
        ),
    )
    .expect("write config.toml");

    let server = start_mock_server().await;
    let output = test
        .cmd_with_server(&server)
        .env("PATH", git.path_env())
        .arg("--skip-git-repo-check")
        .arg("-")
        .write_stdin("x".repeat(MAX_USER_INPUT_TEXT_CHARS + 1))
        .output()
        .expect("run codex-exec");

    // Report only the stderr tail: the summary echoes the 1 MiB prompt.
    let stderr = String::from_utf8_lossy(&output.stderr);
    let tail = &stderr[stderr.floor_char_boundary(stderr.len().saturating_sub(2000))..];
    assert!(!output.status.success(), "exec succeeded: {tail}");
    assert!(stderr.contains("turn/start"), "{tail}");
    git.assert_stopped(test.home_path());
}
