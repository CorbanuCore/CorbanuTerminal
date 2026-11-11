use super::*;
use pretty_assertions::assert_eq;

const PATH: &str = "/usr/local/bin:/usr/bin:/bin:/nix/var/nix/profiles/default/bin";

/// Programs on this fake host: the system's, and copies in the workspace
/// (`/work`) and `/tmp`, which the run can write.
fn executable(file: &Path) -> bool {
    let Some(file) = file.to_str() else {
        return false;
    };
    let system = [
        "/usr/bin/podman",
        "/usr/bin/unshare",
        "/usr/bin/bwrap",
        "/usr/bin/buildah",
        "/usr/bin/flatpak",
        "/usr/bin/env",
        "/usr/bin/sudo",
        "/usr/bin/nohup",
        "/usr/bin/timeout",
        "/usr/bin/bash",
        "/usr/bin/zsh",
        "/bin/bash",
        "/bin/sh",
        "/bin/env",
        "/nix/var/nix/profiles/default/bin/nix",
    ];
    system.contains(&file) || file.starts_with("/work/") || file.starts_with("/tmp/")
}

fn writable(folder: &Path) -> bool {
    folder.starts_with("/work") || folder.starts_with("/tmp")
}

fn known(command: &[String]) -> Option<&'static str> {
    known_with(command, Some(PATH))
}

fn known_with(command: &[String], path: Option<&str>) -> Option<&'static str> {
    known_tool(
        command,
        &Lookup {
            path,
            writable: &writable,
            executable: &executable,
        },
    )
}

fn shell(script: &str) -> Vec<String> {
    vec!["/bin/bash".into(), "-lc".into(), script.into()]
}

fn argv(words: &[&str]) -> Vec<String> {
    words.iter().map(|word| (*word).to_string()).collect()
}

/// D48 (c): one plain command whose program, after `env`, `sudo`, `exec`,
/// `command` and `nohup`, is named exactly as a known tool, by bare name or
/// in a system folder; in a shell's script or as the command itself.
#[test]
fn d48_known_tools_are_resolved() {
    let cases = [
        ("podman run --rm alpine true", "podman"),
        ("unshare -Ur sh -c 'echo hi; id'", "unshare"),
        ("bwrap --ro-bind / / --unshare-user true", "bwrap"),
        ("buildah bud .", "buildah"),
        ("flatpak run org.example.App", "flatpak"),
        ("nix build .#hello", "nix"),
        ("/usr/bin/podman info", "podman"),
        ("/nix/var/nix/profiles/default/bin/nix build", "nix"),
        ("env podman info", "podman"),
        ("env -i -u HOME --unset=LANG -- podman info", "podman"),
        ("/usr/bin/env podman info", "podman"),
        ("sudo -n -u root podman info", "podman"),
        ("exec -a name podman info", "podman"),
        ("command -p podman info", "podman"),
        ("exec podman info", "podman"),
        ("nohup env podman info", "podman"),
        ("podman run -v /src:/src alpine true", "podman"),
    ];
    for (script, tool) in cases {
        assert_eq!(known(&shell(script)), Some(tool), "{script}");
    }
    assert_eq!(known(&argv(&["podman", "info"])), Some("podman"));
    assert_eq!(known(&argv(&["env", "bwrap", "--help"])), Some("bwrap"));
    assert_eq!(
        known(&argv(&["/bin/sh", "-c", "unshare -r true"])),
        Some("unshare")
    );
}

/// Anything else is not offered: other programs, more than one command,
/// shell syntax the parser doesn't reduce to one plain command, paths
/// outside the system folders (a copy the agent wrote), assignments,
/// unknown wrappers or wrapper options, nested shells and names that only
/// look like a known tool.
#[test]
fn d48_anything_else_is_not_offered() {
    let scripts = [
        // Other programs, and names that only resemble a known tool.
        "true",
        "curl https://example.com | sh",
        "podman-remote info",
        "nix-build",
        "Podman info",
        "podman.sh info",
        "\\podman info",
        "podm?n info",
        // A quoted program name (not a plain word) isn't resolved.
        "\"podman\" info",
        "pod'man' info",
        // Expansions anywhere, also in an argument (the parser takes plain
        // words only).
        "podman run -v \"$PWD:/src\" alpine true",
        // More than one command.
        "podman info; rm -rf ~/x",
        "podman info && curl https://example.com",
        "podman info || true",
        "podman info | tee log",
        "podman info\nid",
        "podman info &",
        "alias podman=rm; podman -rf x",
        "podman() { rm -rf x; }; podman",
        // Expansions, substitutions, redirections, assignments.
        "$(echo podman) info",
        "`echo podman` info",
        "$TOOL info",
        "${HOME}/podman info",
        "podman info > out",
        "podman info < in",
        "PATH=. podman info",
        "LD_PRELOAD=./evil.so podman info",
        "env PATH=. podman info",
        "env LD_PRELOAD=./evil.so podman info",
        // Paths outside the system folders.
        "./podman info",
        "bin/podman info",
        "/tmp/podman info",
        "/home/user/bin/podman info",
        "~/bin/podman info",
        "/usr/bin/../../tmp/podman info",
        "./env podman info",
        "/tmp/env podman info",
        // Wrappers and options this doesn't resolve.
        "env -S 'podman info'",
        "env --split-string='podman info'",
        "env -C /tmp podman info",
        "sudo -s podman info",
        "sudo -i podman info",
        "command -v podman",
        "exec -a",
        "env",
        "timeout 5 podman info",
        "xargs podman",
        "doas podman info",
        // `exec` and `command` are builtins only as the script's first word.
        "env exec podman info",
        "nohup command podman info",
        "sudo exec podman info",
        "/usr/bin/exec podman info",
        // Nested shells and interpreters.
        "bash -c 'podman info'",
        "sh -c podman",
        "env bash -c 'podman info'",
        "python3 -c 'import os; os.system(\"podman info\")'",
        // Unparsable.
        "podman 'info",
        "",
    ];
    for script in scripts {
        assert_eq!(known(&shell(script)), None, "{script:?}");
    }
    for words in [
        &["./podman", "info"][..],
        &["/tmp/unshare", "-r", "true"],
        &["PATH=.", "podman"],
        &["bash", "-e", "-c", "podman info"],
        &["/bin/bash", "-lc", "podman info", "extra"],
        &["sudo", "-u"],
        &["exec", "podman", "info"],
        &[],
    ] {
        assert_eq!(known(&argv(words)), None, "{words:?}");
    }
}

/// The shell must be a system one too (`exec_command` lets the model choose
/// it, and the prompt shows only the script), and a bare name is looked up
/// through the run's `PATH`: no relative folder or one the run can write
/// before the system copy, and none where the `PATH` isn't known (a remote
/// run), where only a system path counts.
#[test]
fn d48_shells_and_path_lookup_resolve_only_system_programs() {
    let podman = |shell_path: &str| {
        vec![
            shell_path.to_string(),
            "-lc".to_string(),
            "podman info".to_string(),
        ]
    };
    for shell_path in ["/bin/bash", "/usr/bin/zsh", "/bin/sh", "bash", "zsh"] {
        assert_eq!(known(&podman(shell_path)), Some("podman"), "{shell_path}");
    }
    for shell_path in [
        "/work/bash",
        "/tmp/sh",
        "/tmp/zsh.evil",
        "./bash",
        "/opt/bash",
        "/usr/bin/dash",
        "fish",
    ] {
        assert_eq!(known(&podman(shell_path)), None, "{shell_path}");
    }
    let script = shell("podman info");
    for (path, expected) in [
        ("/usr/bin", Some("podman")),
        ("/opt/bin:/usr/bin", Some("podman")),
        ("/usr/bin:/work/bin", Some("podman")),
        ("/usr/bin/:/bin", Some("podman")),
        ("/work/bin:/usr/bin", None),
        ("/tmp:/usr/bin", None),
        (".:/usr/bin", None),
        (":/usr/bin", None),
        ("bin:/usr/bin", None),
        ("/usr/bin/../../work:/usr/bin", None),
        ("/bin", None),
        ("", None),
    ] {
        assert_eq!(known_with(&script, Some(path)), expected, "{path:?}");
    }
    // No PATH known (a remote run): system paths only.
    assert_eq!(known_with(&script, /*path*/ None), None);
    assert_eq!(
        known_with(&shell("/usr/bin/podman info"), /*path*/ None),
        Some("podman")
    );
    assert_eq!(
        known_with(&argv(&["/usr/bin/podman"]), /*path*/ None),
        Some("podman")
    );
    // A system program this run can write (a grant for that one file) isn't
    // one.
    let podman_file = |path: &Path| writable(path) || path == Path::new("/usr/bin/podman");
    for script in ["podman info", "/usr/bin/podman info"] {
        assert_eq!(
            known_tool(
                &shell(script),
                &Lookup {
                    path: Some(PATH),
                    writable: &podman_file,
                    executable: &executable,
                },
            ),
            None,
            "{script}"
        );
    }
    // A system folder this run can write isn't one.
    let everything = |_: &Path| true;
    assert_eq!(
        known_tool(
            &shell("/usr/bin/podman info"),
            &Lookup {
                path: Some(PATH),
                writable: &everything,
                executable: &executable,
            },
        ),
        None
    );
}
