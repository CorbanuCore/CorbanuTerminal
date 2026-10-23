use super::*;
#[cfg(unix)]
use codex_protocol::models::PermissionProfile;
#[cfg(unix)]
use codex_protocol::permissions::NetworkSandboxPolicy;
use pretty_assertions::assert_eq;

fn shell(script: &str) -> Vec<String> {
    vec!["/bin/zsh".into(), "-lc".into(), script.into()]
}

fn argv(words: &[&str]) -> Vec<String> {
    words.iter().map(|word| (*word).to_string()).collect()
}

fn targets(command: &[String]) -> Vec<String> {
    link_targets(command)
        .candidates
        .into_iter()
        .map(|candidate| candidate.target)
        .collect()
}

#[test]
fn issue_623_finds_what_ln_cp_and_windows_links_point_to() {
    assert_eq!(targets(&shell("ln -s .agents ag")), vec![".agents"]);
    assert_eq!(
        targets(&shell("ln -s .agents ag && echo x >> ag/notes.md")),
        vec![".agents"]
    );
    assert_eq!(targets(&shell("ln -sfn '/p/.codex' x")), vec!["/p/.codex"]);
    assert_eq!(
        targets(&shell("ln --symbolic -- .agents ag")),
        vec![".agents"]
    );
    assert_eq!(targets(&shell("ln -s .agents")), vec![".agents"]);
    assert_eq!(
        targets(&shell("ln -s -t links .codex .agents")),
        vec![".codex", ".agents"]
    );
    assert_eq!(
        targets(&shell("ln .agents/notes.md h")),
        vec![".agents/notes.md"]
    );
    assert_eq!(
        targets(&shell("cd sub; ln -s ../.agents a")),
        vec!["../.agents"]
    );
    assert_eq!(
        targets(&shell("cp -s .agents/notes.md x")),
        vec![".agents/notes.md"]
    );
    assert_eq!(targets(&shell("cp -r .agents x")), Vec::<String>::new());
    assert_eq!(
        targets(&shell("bash -c 'sudo ln -s .codex c'")),
        vec![".codex"]
    );
    assert_eq!(
        targets(&argv(&["cmd.exe", "/c", "mklink", "/D", "ag", ".agents"])),
        vec![".agents"]
    );
    assert_eq!(
        targets(&argv(&[
            "powershell.exe",
            "-Command",
            "New-Item -ItemType SymbolicLink -Path ag -Target .agents"
        ])),
        vec![".agents"]
    );
    assert_eq!(
        targets(&shell("ls -la ag; cat notes.txt")),
        Vec::<String>::new()
    );
    // The link's own name is not a target.
    let found = link_targets(&shell("ln -s .agents sub/ag")).candidates;
    assert_eq!(
        found[0].link_folders,
        vec!["sub".to_string(), "sub/ag".to_string()]
    );
}

#[test]
fn issue_623_finds_links_made_by_interpreters() {
    assert_eq!(
        targets(&shell(
            "python3 -c \"import os; os.symlink('.agents', 'ag')\""
        )),
        vec![".agents", "ag"]
    );
    assert_eq!(
        targets(&argv(&[
            "python3.12",
            "-c",
            "import pathlib; pathlib.Path('ag').symlink_to('/p/.codex')"
        ])),
        vec!["ag", "/p/.codex"]
    );
    assert_eq!(
        targets(&argv(&[
            "node",
            "-e",
            "require('fs').symlinkSync(\".agents\", 'x')"
        ])),
        vec!["fs", ".agents", "x"]
    );
    assert_eq!(
        targets(&shell("perl -e 'symlink(\".agents\", \"x\")'")),
        vec![".agents", "x"]
    );
    assert!(
        targets(&shell(
            "python3 - <<'EOF'\nimport os\nos.symlink('.agents', 'ag')\nEOF"
        ))
        .contains(&".agents".to_string())
    );
    assert!(
        targets(&shell(
            "python3 -c \"import os\nos.symlink('.agents', 'ag')\""
        ))
        .contains(&".agents".to_string())
    );
    assert_eq!(targets(&shell("x\nln -s .agents ag")), vec![".agents"]);
    // Code that makes no link names nothing.
    assert_eq!(
        targets(&shell(
            "python3 -c \"print(open('.agents/notes.md').read())\""
        )),
        Vec::<String>::new()
    );
}

/// Review r1: commands behind shell keywords, grouping and wrappers, `-t`
/// in an option cluster, heredocs behind a wrapper, and interpreters that
/// run `ln` themselves.
#[test]
fn issue_623_finds_links_behind_keywords_wrappers_and_interpreter_commands() {
    for script in [
        "if [ -d .agents ]; then ln -s .agents ag; fi",
        "for f in x; do ln -s .agents ag; done",
        "(cd sub && ln -s ../.agents a)",
        "{ ln -s .agents ag; }",
        "x=$(ln -s .agents ag)",
        "sudo -u root ln -s .agents ag",
        "env -i PATH=/bin ln -s .agents ag",
        "nice -n5 ln -s .agents ag",
        "timeout 5 ln -s .agents ag",
        "doas ln -s .agents ag",
        "echo ag | xargs ln -s .agents",
        "find . -maxdepth 0 -exec ln -s .agents ag \\;",
        "! ln -s .agents ag",
    ] {
        assert!(
            targets(&shell(script))
                .iter()
                .any(|target| target.ends_with(".agents")),
            "{script}: {:?}",
            targets(&shell(script))
        );
    }
    assert_eq!(targets(&shell("ln -st links .agents")), vec![".agents"]);
    assert_eq!(
        link_targets(&shell("ln -sft links .agents")).candidates[0].link_folders,
        vec!["links".to_string()]
    );
    assert_eq!(targets(&shell("ln -tlinks -s .codex")), vec![".codex"]);
    assert_eq!(
        targets(&shell("ln --target-directory=links -s .codex")),
        vec![".codex"]
    );
    for script in [
        "sudo python3 - <<'EOF'\nimport os\nos.symlink('.agents', 'ag')\nEOF",
        "/usr/bin/env python3 - <<EOF\nimport os\nos.symlink('.agents', 'ag')\nEOF",
        "uv run python - <<EOF\nimport os\nos.symlink('.agents', 'ag')\nEOF",
        "python3 -c \"import os; os.system('ln -s .agents ag')\"",
        "python3 -c \"import subprocess; subprocess.run(['ln', '-s', '.agents', 'ag'])\"",
        "node -e \"require('child_process').execSync('ln -s .agents ag')\"",
        "ruby -e \"require 'fileutils'; FileUtils.ln_s('.agents', 'ag')\"",
    ] {
        assert!(
            targets(&shell(script)).contains(&".agents".to_string()),
            "{script}: {:?}",
            targets(&shell(script))
        );
    }
    assert_eq!(
        targets(&argv(&[
            "cmd",
            "/c",
            "cd",
            "sub",
            "&&",
            "mklink",
            "/D",
            "ag",
            "..\\.agents"
        ])),
        vec!["..\\.agents"]
    );
}

/// Review r2: operators inside words, line continuations, `cp` options
/// before `-s`, functions, Ruby without parentheses, PowerShell's .NET call,
/// GNU `gln`, and a bound that refuses instead of dropping.
#[test]
fn issue_623_finds_links_behind_operators_continuations_and_more_tools() {
    for script in [
        "true;ln -s .agents ag",
        "cd sub&&ln -s ../.agents x",
        "ls||ln -s .agents ag",
        "ln -s \\\n  .agents ag",
        "cp --verbose -s .agents x",
        "f(){ ln -s .agents ag; }; f",
        "f() { ln -s .agents ag; }",
        "ruby -e \"require 'fileutils'; FileUtils.ln_s '.agents', 'ag'\"",
        "gln -s .agents ag",
    ] {
        assert!(
            targets(&shell(script))
                .iter()
                .any(|target| target.ends_with(".agents")),
            "{script}: {:?}",
            targets(&shell(script))
        );
    }
    assert!(
        targets(&argv(&[
            "pwsh",
            "-Command",
            "[IO.File]::CreateSymbolicLink('ag', '.agents')"
        ]))
        .contains(&".agents".to_string())
    );
    // Quoted operators stay in their word.
    assert_eq!(
        targets(&shell("echo 'a;ln -s .agents ag' > notes.txt")),
        Vec::<String>::new()
    );
    let many = (0..300)
        .map(|index| format!("'x{index}'"))
        .collect::<Vec<_>>()
        .join(", ");
    let code = format!("import os; [{many}]; os.symlink('.agents', 'ag')");
    assert!(link_targets(&argv(&["python3", "-c", &code])).overflow);
    // Many strings in code that makes no link are not a reason to refuse.
    let quiet = format!("print([{many}])");
    assert!(!link_targets(&argv(&["python3", "-c", &quiet])).overflow);
}

/// Review r3: a long first string must not hide a link call (the bound
/// refuses even when nothing was collected).
#[test]
fn issue_623_overflow_without_candidates_still_counts() {
    let code = format!(
        "x='{}'; import os; os.symlink('.agents', 'ag')",
        "a".repeat(5000)
    );
    let command = argv(&["python3", "-c", &code]);
    let found = link_targets(&command);
    assert!(found.overflow, "{found:?}");
    assert!(may_link_command(&command));
    assert!(!may_link_command(&shell("cat .agents/notes.md")));
    assert!(!may_link_command(&shell("ls -la")));
}

#[test]
fn issue_623_only_calls_that_create_a_link_count() {
    for code in [
        "os.symlink('a', 'b')",
        "fs.symlinkSync('a', 'b')",
        "Path('b').symlink_to('a')",
        "os.link('a', 'b')",
        "symlink \"a\", \"b\"",
        "File.link('a', 'b')",
    ] {
        assert!(calls_link(code), "{code}");
    }
    for code in [
        "os.unlink('a')",
        "os.readlink('a')",
        "Path('a').is_symlink()",
        "os.path.islink('a')",
        "print('symlinks')",
    ] {
        assert!(!calls_link(code), "{code}");
    }
    assert_eq!(
        targets(&shell("python3 -c \"import os; os.unlink('.agents/x')\"")),
        Vec::<String>::new()
    );
}

#[test]
fn issue_623_reads_relative_targets_from_folders_the_script_changes_to() {
    let found = link_targets(&shell("cd sub && ln -s ../.agents x")).candidates;
    assert_eq!(found.len(), 1);
    assert!(
        found[0].link_folders.contains(&"sub".to_string()),
        "{found:?}"
    );
}

#[cfg(unix)]
struct Project {
    _home: tempfile::TempDir,
    _root: tempfile::TempDir,
    root: PathBuf,
    inputs: Inputs,
}

#[cfg(unix)]
fn project() -> Project {
    let home = tempfile::TempDir::new().expect("home");
    let root_dir = tempfile::TempDir::new().expect("project");
    let root = root_dir.path().canonicalize().expect("real project");
    std::fs::create_dir_all(root.join(".agents")).expect(".agents");
    std::fs::create_dir_all(root.join("sub")).expect("sub");
    let turn_cwd = AbsolutePathBuf::from_absolute_path(&root).expect("absolute");
    let inputs = Inputs {
        codex_home: home.path().canonicalize().expect("real home"),
        sqlite_home: home.path().canonicalize().expect("real home"),
        turn_cwd: turn_cwd.clone(),
        workspace_roots: vec![turn_cwd],
        profile: PermissionProfile::workspace_write_with(
            &[],
            NetworkSandboxPolicy::Restricted,
            /*exclude_tmpdir_env_var*/ true,
            /*exclude_slash_tmp*/ true,
        ),
        user_home: None,
    };
    Project {
        _home: home,
        _root: root_dir,
        root,
        inputs,
    }
}

#[cfg(unix)]
/// What [`refusal`] would name for `command`, without a session.
fn refused(project: &Project, command: &[String], cwd: &Path) -> Option<String> {
    if !may_link_command(command) {
        return None;
    }
    project.inputs.settings_named(command, cwd).or_else(|| {
        project
            .inputs
            .protected_target(&link_targets(command).candidates, cwd)
            .map(|refused| refused.target)
    })
}

#[cfg(unix)]
#[test]
fn issue_623_refuses_relative_and_absolute_links_into_agent_settings() {
    let project = project();
    let root = project.root.as_path();
    let absolute = format!("ln -s {}/.codex c", root.display());
    assert_eq!(
        refused(&project, &shell("ln -s .agents ag"), root),
        Some(".agents".to_string())
    );
    assert_eq!(
        refused(&project, &shell(&absolute), root),
        Some(".codex".to_string())
    );
    // Relative to the link's folder, and from a folder below the project.
    assert_eq!(
        refused(&project, &shell("ln -s ../.agents sub/ag"), root),
        Some(".agents".to_string())
    );
    assert_eq!(
        refused(&project, &shell("ln -s ../.agents ag"), &root.join("sub")),
        Some(".agents".to_string())
    );
    assert_eq!(
        refused(
            &project,
            &shell("python3 -c \"import os; os.symlink('.agents', 'ag')\""),
            root
        ),
        Some(".agents".to_string())
    );
    assert_eq!(
        refused(&project, &shell("cd sub && ln -s ../.agents x"), root),
        Some(".agents".to_string())
    );
    // `$PWD` is the shell's folder.
    assert_eq!(
        refused(&project, &shell("ln -s \"$PWD/.agents\" ag"), root),
        Some(".agents".to_string())
    );
    assert_eq!(
        refused(&project, &shell("ln -s \"$(pwd)/.codex\" ag"), root),
        Some(".codex".to_string())
    );
    // From `sub`, `evil/../.agents` reads as `sub/.agents`, but `evil`
    // links to `inner`, so the system reads the project's `.agents`.
    std::fs::create_dir_all(root.join("inner")).expect("inner");
    std::os::unix::fs::symlink(root.join("inner"), root.join("sub/evil")).expect("evil");
    assert_eq!(
        refused(
            &project,
            &shell("ln -s evil/../.agents x"),
            &root.join("sub")
        ),
        Some(".agents".to_string())
    );
    // Two steps: a link whose text names `.agents` anywhere is refused.
    assert_eq!(
        refused(&project, &shell("ln -s y/.agents x"), root),
        Some(".agents".to_string())
    );
    assert_eq!(
        refused(&project, &shell("ln -s .agen* ag"), root),
        Some(".agen*".to_string())
    );
    // Review r3: substitutions, variables, `eval`, pipes into a shell and
    // braces name `.agents` too, and are refused.
    for script in [
        "echo $(ln -s .agents ag)",
        "true \"$(ln -s .agents ag)\"",
        "ls `ln -s .agents ag`",
        "cat <(ln -s .agents ag)",
        "eval 'ln -s .agents ag'",
        "echo 'ln -s .agents ag' | sh",
        "sh <<< 'ln -s .agents ag'",
        "ln -s \"$(realpath .agents)\" ag",
        "ln -s \"${X:-.agents}\" ag",
        "A=.agents; ln -s \"$A\" ag",
        "mkdir d; ln -s ../.agent{s,} d",
    ] {
        assert!(
            project
                .inputs
                .settings_named(&shell(script), root)
                .is_some(),
            "{script}"
        );
    }
    // A project kept below a folder named `.codex` is not one.
    let inside = format!("ln -s {}/notes.txt n", root.display());
    assert_eq!(project.inputs.settings_named(&shell(&inside), root), None);
    // Folders a script changes to are expanded like targets.
    let home = format!(
        "cd {} && ln -s rules r",
        project.inputs.codex_home.display()
    );
    assert!(refused(&project, &shell(&home), root).is_some());
    // A target that leads there through an existing link (the text names
    // no protected folder): resolved as the system reads it.
    std::os::unix::fs::symlink(root.join(".agents"), root.join("al")).expect("al");
    assert_eq!(
        refused(&project, &shell("ln -s al x"), root),
        Some("al".to_string())
    );
    std::fs::create_dir_all(root.join("inner2")).expect("inner2");
    std::os::unix::fs::symlink(root.join("inner2"), root.join("sub/up")).expect("up");
    assert_eq!(
        refused(&project, &shell("ln -s up/../al x"), &root.join("sub")),
        Some("up/../al".to_string())
    );
    // A link to the Corbanu home is refused too.
    let home = format!("ln -s {} h", project.inputs.codex_home.display());
    assert!(refused(&project, &shell(&home), root).is_some());
}

#[cfg(unix)]
#[test]
fn issue_623_other_links_are_not_refused() {
    let project = project();
    let root = project.root.as_path();
    for script in [
        "ln -s src/notes.py n",
        "ln -s .agents-old x",
        "ln -s . self",
        "python3 -c \"import os; os.symlink('notes.txt', 'n')\"",
        "cat .agents/notes.md",
    ] {
        assert_eq!(refused(&project, &shell(script), root), None, "{script}");
    }
}

/// #744: on a loaded machine the lookups took over 2 s and an ordinary
/// command was refused; a lookup that takes 3 s now finishes and decides.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_744_a_slow_lookup_is_waited_for() {
    let found = looked_up(LOOKUP_TIMEOUT, "'{\"hooks\":{}}'".to_string(), || {
        std::thread::sleep(std::time::Duration::from_secs(3));
        None
    })
    .await;
    assert_eq!(found, None);
}

/// #744: lookups that do not finish in time, or fail, still refuse the
/// command, naming the first link target.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_744_lookups_that_do_not_finish_still_refuse() {
    let timeout = std::time::Duration::from_millis(50);
    let slow = looked_up(timeout, ".agents".to_string(), || {
        std::thread::sleep(std::time::Duration::from_millis(500));
        None
    })
    .await;
    // #744 follow-up: and say which check refused it.
    assert_eq!(
        slow,
        Some(Refused {
            target: ".agents".to_string(),
            by: RefusedBy::Timeout
        })
    );
    let failed = looked_up(LOOKUP_TIMEOUT, ".codex".to_string(), || {
        panic!("lookup failed")
    })
    .await;
    assert_eq!(
        failed,
        Some(Refused {
            target: ".codex".to_string(),
            by: RefusedBy::LookupFailed
        })
    );
    let named = Refused {
        target: "x/.agents".to_string(),
        by: RefusedBy::SettingsNamed,
    };
    let found = named.clone();
    let named_found = looked_up(LOOKUP_TIMEOUT, ".codex".to_string(), move || Some(found)).await;
    assert_eq!(named_found, Some(named));
}

/// #744 follow-up (Travis, 2026-10-14): a refusal says which check made it;
/// for a path no grant lifts, where the target landed and which kept path
/// it landed in.
#[cfg(unix)]
#[test]
fn issue_744_a_protected_target_names_where_it_landed() {
    let project = project();
    let root = project.root.as_path();
    let agents = AbsolutePathBuf::from_absolute_path(root.join(".agents")).expect("absolute");
    let refused = project
        .inputs
        .protected_target(&link_targets(&shell("ln -s .agents ag")).candidates, root)
        .expect("refused");
    assert_eq!(refused.target, ".agents");
    assert_eq!(
        refused.by,
        RefusedBy::ProtectedTarget {
            landed: agents.clone(),
            kept: agents.clone(),
        }
    );
    // Through an existing link: it lands where the link points.
    std::os::unix::fs::symlink(root.join(".agents"), root.join("al")).expect("al");
    let refused = project
        .inputs
        .protected_target(&link_targets(&shell("ln -s al x")).candidates, root)
        .expect("refused");
    assert_eq!(refused.target, "al");
    assert_eq!(
        refused.by,
        RefusedBy::ProtectedTarget {
            landed: agents.clone(),
            kept: agents,
        }
    );
}

/// #744 follow-up: the refusal is logged with its check and, for a protected
/// target, the paths; the command's text is not.
#[test]
fn issue_744_the_refusal_is_logged_with_its_check() {
    let buffer: &'static std::sync::Mutex<Vec<u8>> =
        Box::leak(Box::new(std::sync::Mutex::new(Vec::new())));
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(tracing_test::internal::MockWriter::new(buffer))
        .finish();
    let root = std::env::temp_dir();
    let landed = AbsolutePathBuf::from_absolute_path(root.join("p/.agents/x")).expect("absolute");
    let kept = AbsolutePathBuf::from_absolute_path(root.join("p/.agents")).expect("absolute");
    tracing::subscriber::with_default(subscriber, || {
        log_refusal(
            &Refused {
                target: "'{\"hooks\":{}}'".to_string(),
                by: RefusedBy::ProtectedTarget {
                    landed: landed.clone(),
                    kept: kept.clone(),
                },
            },
            std::time::Duration::from_millis(1234),
        );
        log_refusal(
            &Refused {
                target: ".agents".to_string(),
                by: RefusedBy::Timeout,
            },
            std::time::Duration::from_secs(10),
        );
    });
    let logs = String::from_utf8(buffer.lock().expect("buffer").clone()).expect("utf8");
    let lines: Vec<&str> = logs.lines().collect();
    assert_eq!(lines.len(), 2, "{logs}");
    let protected = format!(
        "refused_by=protected_target target='{{\"hooks\":{{}}}}' landed={} kept={} elapsed_ms=1234",
        landed.as_path().display(),
        kept.as_path().display()
    );
    let timeout = "refused_by=timeout target=.agents elapsed_ms=10000".to_string();
    for (line, detail) in lines.iter().zip([&protected, &timeout]) {
        assert!(line.contains(" WARN "), "{line}");
        assert!(
            line.ends_with(&format!(
                "Aggressive link check (#623) refused a command: {detail}"
            )),
            "{line}"
        );
    }
    // Kept for tests to report (other tests may add their own).
    let recent = recent_refusals();
    assert!(recent.contains(&protected), "{recent:?}");
    assert!(recent.contains(&timeout), "{recent:?}");
}

/// #744 follow-up (review): the log shows a command-derived target as the
/// refusal does: control characters escaped and at most 200 characters.
#[test]
fn issue_744_the_logged_target_is_escaped_and_bounded() {
    let buffer: &'static std::sync::Mutex<Vec<u8>> =
        Box::leak(Box::new(std::sync::Mutex::new(Vec::new())));
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(tracing_test::internal::MockWriter::new(buffer))
        .finish();
    let target = format!("x\nWARN forged line\u{1b}[31m{}", "a".repeat(300));
    let landed =
        AbsolutePathBuf::from_absolute_path(std::env::temp_dir().join(&target)).expect("absolute");
    let kept = AbsolutePathBuf::from_absolute_path(std::env::temp_dir()).expect("absolute");
    tracing::subscriber::with_default(subscriber, || {
        log_refusal(
            &Refused {
                target: target.clone(),
                by: RefusedBy::ProtectedTarget { landed, kept },
            },
            std::time::Duration::from_millis(1),
        );
    });
    let logs = String::from_utf8(buffer.lock().expect("buffer").clone()).expect("utf8");
    assert_eq!(logs.lines().count(), 1, "{logs}");
    assert!(!logs.contains('\u{1b}'), "{logs}");
    assert!(
        logs.contains(&format!("target={}", shown_path(&target))),
        "{logs}"
    );
    assert!(shown_path(&target).chars().count() <= 201);
    assert!(!logs.contains(&"a".repeat(250)), "{logs}");
}
