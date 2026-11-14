//! #1805: Windows environment names ignore case. A caller's `Path` (the
//! spelling `std::env::vars()` gives on Windows) is the same variable as the
//! sandbox's `PATH`, so a command must never get both.

use super::*;
use crate::process::make_env_block;
use pretty_assertions::assert_eq;

fn env(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

fn spellings(env_map: &HashMap<String, String>, name: &str) -> Vec<String> {
    let mut keys: Vec<String> = env_map
        .keys()
        .filter(|key| key.eq_ignore_ascii_case(name))
        .cloned()
        .collect();
    keys.sort();
    keys
}

/// The entries of an environment block, in order.
fn block_entries(block: &[u16]) -> Vec<String> {
    String::from_utf16(block)
        .expect("UTF-16 block")
        .split('\0')
        .filter(|entry| !entry.is_empty())
        .map(str::to_string)
        .collect()
}

#[test]
fn inherit_path_env_keeps_the_callers_path_in_any_spelling() {
    let mut env_map = env(&[("Path", r"C:\caller"), ("PathExt", ".EXE")]);

    inherit_path_env(&mut env_map);

    assert_eq!(spellings(&env_map, "PATH"), vec!["Path".to_string()]);
    assert_eq!(spellings(&env_map, "PATHEXT"), vec!["PathExt".to_string()]);
    assert_eq!(env_map.get("Path"), Some(&r"C:\caller".to_string()));
    assert_eq!(env_map.get("PathExt"), Some(&".EXE".to_string()));
}

#[test]
fn inherit_path_env_adds_the_parents_path_when_the_caller_has_none() {
    let mut env_map = env(&[("TEMP", r"C:\temp")]);

    inherit_path_env(&mut env_map);

    assert_eq!(env_map.get("PATH").cloned(), std::env::var("PATH").ok());
}

#[test]
fn prepend_path_extends_the_callers_path_and_replaces_its_spelling() {
    let mut env_map = env(&[("Path", r"C:\caller"), ("TEMP", r"C:\temp")]);

    prepend_path(&mut env_map, r"C:\denybin");

    assert_eq!(
        env_map,
        env(&[("PATH", r"C:\denybin;C:\caller"), ("TEMP", r"C:\temp")])
    );
}

#[test]
fn reorder_pathext_reads_and_replaces_any_spelling() {
    let mut env_map = env(&[("PathExt", ".EXE;.BAT;.CMD")]);

    reorder_pathext_for_stubs(&mut env_map);

    assert_eq!(env_map, env(&[("PATHEXT", ".BAT;.CMD;.EXE")]));
}

#[test]
fn get_ignoring_case_prefers_the_spelling_the_block_keeps() {
    let env_map = env(&[("path", "lower"), ("Path", "mixed"), ("PATH", "upper")]);

    assert_eq!(
        get_ignoring_case(&env_map, "Path"),
        Some(&"upper".to_string())
    );
    assert_eq!(get_ignoring_case(&env_map, "PATHEXT"), None);
}

#[test]
fn make_env_block_holds_each_name_once() {
    let env_map = env(&[
        ("Path", "mixed"),
        ("path", "lower"),
        ("PATH", "upper"),
        ("Temp", r"C:\temp"),
        ("ComSpec", "cmd.exe"),
    ]);

    let block = make_env_block(&env_map);

    assert_eq!(
        block_entries(&block),
        vec![
            "ComSpec=cmd.exe".to_string(),
            "PATH=upper".to_string(),
            r"Temp=C:\temp".to_string(),
        ]
    );
    assert_eq!(&block[block.len() - 2..], &[0, 0]);
}

/// #2155: refuse every transport, including local-file submodule clones,
/// over a caller's value in any spelling.
#[test]
fn no_network_env_refuses_all_git_transports() {
    let mut env_map = env(&[("Git_Allow_Protocol", "ext:https")]);

    apply_no_network_env_vars(&mut env_map);

    assert_eq!(
        spellings(&env_map, "GIT_ALLOW_PROTOCOL"),
        vec!["GIT_ALLOW_PROTOCOL".to_string()]
    );
    assert_eq!(env_map.get("GIT_ALLOW_PROTOCOL"), Some(&String::new()));
    assert_eq!(
        spellings(&env_map, "GIT_ALLOW_PROTOCOLS"),
        Vec::<String>::new()
    );
}

/// #2155: real git must refuse network and file transports even when config
/// permits them. Local repository operations still work.
#[test]
fn no_network_env_git_refuses_network_and_local_submodule_clones() {
    let mut vars = HashMap::new();
    apply_no_network_env_vars(&mut vars);
    let dir = tempfile::tempdir().expect("tempdir");
    let git = |args: &[&str], extra_env: &HashMap<String, String>| {
        let mut command = std::process::Command::new("git");
        for (key, _) in std::env::vars_os() {
            if key
                .to_string_lossy()
                .to_ascii_uppercase()
                .starts_with("GIT_")
            {
                command.env_remove(key);
            }
        }
        command
            .args(args)
            .current_dir(dir.path())
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "NUL")
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("GIT_AUTHOR_NAME", "Sandbox test")
            .env("GIT_AUTHOR_EMAIL", "sandbox@example.invalid")
            .env("GIT_COMMITTER_NAME", "Sandbox test")
            .env("GIT_COMMITTER_EMAIL", "sandbox@example.invalid")
            .envs(extra_env)
            .output()
            .expect("run git")
    };
    let setup = |args: &[&str]| {
        let output = git(args, &HashMap::new());
        assert!(output.status.success(), "git {args:?}: {output:?}");
        output
    };
    let refused = |args: &[&str], protocol: &str| {
        let output = git(args, &vars);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success(), "git {args:?}: {output:?}");
        assert!(
            stderr.contains(&format!("transport '{protocol}' not allowed")),
            "git {args:?}: {stderr}"
        );
    };

    for (protocol, url) in [
        ("https", "https://127.0.0.1:9/x.git"),
        ("ssh", "ssh://127.0.0.1:9/x.git"),
        ("git", "git://127.0.0.1:9/x.git"),
        ("ext", "ext::must-not-run"),
    ] {
        let allow = format!("protocol.{protocol}.allow=always");
        refused(&["-c", &allow, "ls-remote", url], protocol);
    }

    setup(&["init", "--quiet", "origin"]);
    setup(&["-C", "origin", "commit", "--allow-empty", "-qm", "fixture"]);
    let origin = dir.path().join("origin");
    let origin = origin.to_str().expect("origin path");
    setup(&["init", "--quiet", "parent"]);
    setup(&[
        "-C",
        "parent",
        "-c",
        "protocol.file.allow=always",
        "submodule",
        "add",
        origin,
        "sub",
    ]);
    setup(&["-C", "parent", "commit", "-qam", "local submodule"]);
    setup(&["clone", "--quiet", "parent", "update"]);

    refused(
        &[
            "-C",
            "update",
            "submodule",
            "update",
            "--init",
            "--recursive",
        ],
        "file",
    );
    refused(
        &["clone", "--recurse-submodules", "parent", "recursive"],
        "file",
    );
    let url = format!("file:///{}", origin.replace('\\', "/"));
    refused(
        &["-c", "protocol.file.allow=always", "clone", &url, "copy"],
        "file",
    );
    refused(&["clone", origin, "local-copy"], "file");

    let status = git(&["-C", "parent", "status", "--porcelain"], &vars);
    assert!(status.status.success(), "git status: {status:?}");
    assert!(status.stdout.is_empty(), "git status: {status:?}");
}
