//! #736, #737 and #740: keychain commands, config writers and grant offers
//! that could never apply under Aggressive.

use super::writes::Writes;
use super::writes::writes;
use super::writes::writes_following_cd;
use super::*;
use pretty_assertions::assert_eq;

fn shell(script: &str) -> Vec<String> {
    vec!["/bin/zsh".into(), "-lc".into(), script.into()]
}

fn known(targets: &[&str]) -> Writes {
    Writes {
        targets: targets.iter().map(ToString::to_string).collect(),
        understood: true,
    }
}

/// #737: the config file a config writer writes, and nothing for a read;
/// #740: whether the text shows every write.
#[test]
fn issue_737_740_config_writers_and_understood_commands() {
    let of = |script: &str| writes(&shell(script));
    for (script, expected) in [
        (
            "git config --global core.hooksPath /tmp/h",
            &["~/.gitconfig"][..],
        ),
        ("git config --global --add a.b c", &["~/.gitconfig"]),
        (
            "git config --global --unset core.hooksPath",
            &["~/.gitconfig"],
        ),
        ("git config set --global user.name x", &["~/.gitconfig"]),
        ("git config --system a.b c", &["/etc/gitconfig"]),
        (
            "git config --file ~/.config/git/config a.b c",
            &["~/.config/git/config"],
        ),
        ("git config --file=cfg a.b c", &["cfg"]),
        ("git config core.hooksPath /tmp", &[".git/config"]),
        ("git -C sub config a.b c", &["sub/.git/config"]),
        ("git --git-dir=repo.git config a.b c", &["repo.git/config"]),
        ("npm config set registry https://x", &["~/.npmrc"]),
        ("npm set registry https://x", &["~/.npmrc"]),
        ("npm config set --location=project a b", &[".npmrc"]),
        ("npm config set --userconfig ./rc a b", &["./rc"]),
        (
            "pip config set global.index-url https://x",
            &["~/.config/pip/pip.conf"],
        ),
        (
            "python3 -m pip config unset global.index-url",
            &["~/.config/pip/pip.conf"],
        ),
        ("gh config set editor vim", &["~/.config/gh/config.yml"]),
        // Reads write nothing.
        ("git config --global --get user.name", &[]),
        ("git config --global user.name", &[]),
        ("git config --list --show-origin", &[]),
        ("git config get --global user.name", &[]),
        ("npm config get registry", &[]),
        ("pip config list", &[]),
        ("gh config get editor", &[]),
        // #740: commands whose text shows every write.
        (
            "echo t04 >> ../outside-notes.txt",
            &["../outside-notes.txt"],
        ),
        (
            "printf '#!/bin/sh\\n' > .git/hooks/pre-commit",
            &[".git/hooks/pre-commit"],
        ),
        ("mkdir .agents", &[".agents"]),
        ("cat .codex/notes.md", &[]),
        (
            "ln -s notes.txt notes-link.txt && cat notes-link.txt",
            &["notes-link.txt"],
        ),
        ("ls -la | grep x > out 2>&1", &["out"]),
        ("find . -name '*.rs'", &[]),
        // #789: waiting, and a sed script that only prints lines.
        ("sleep 60", &[]),
        ("sleep 60; echo finished > ks-child.txt", &["ks-child.txt"]),
        ("sed -n 1p notes.txt", &[]),
        ("sed -n '1,240p' /x/skills/SKILL.md", &[]),
        ("sed -ne 1p -e 2d notes.txt", &[]),
        ("sed -n -e '2,$' -e p notes.txt", &[]),
        ("sed -n = notes.txt", &[]),
        ("sed -n -e '$!N' -e 1d notes.txt", &[]),
        ("sed --quiet --expression=5q notes.txt", &[]),
        // #1182: curl without an option that writes a file, and `command
        // -v`.
        ("curl -sS -I https://example.com", &[]),
        (
            "curl -fsSL -H 'Accept: x' -X POST -d @body.json https://x",
            &[],
        ),
        ("curl -sSHx --max-time 5 https://x", &[]),
        ("curl -sS https://x > /tmp/probe1.out", &["/tmp/probe1.out"]),
        ("command -v corbanu", &[]),
        ("command -V git && command -pv ls", &[]),
    ] {
        assert_eq!(of(script), known(expected), "{script}");
    }
    // Not understood: the text may hide a write.
    for script in [
        "./install-hooks.sh",
        "npx husky install",
        "git commit -m x",
        "npm config set -g a b",
        "pip config set --site a b",
        "cd .husky && printf x > pre-commit",
        "echo $(cat x) > notes.txt",
        "echo a\n./build.sh",
        "find . -delete",
        // #789: a sed script that may write, run or read, or comes from a
        // file, or a sed that edits in place.
        "sed -n '1w out.txt' notes.txt",
        "sed 's/a/b/' notes.txt",
        "sed -n -e 1p -e 's/x/y/e' notes.txt",
        "sed 1r/etc/passwd notes.txt",
        "sed -f script.sed notes.txt",
        "sed -ni 1p notes.txt",
        "sed -i '' 1d notes.txt",
        "sed --in-place 1d notes.txt",
        "sed --sandbox 1p notes.txt",
        "sed -n",
        // Review r1: a script a shell expands, an option after an operand
        // (macOS reads it as a file name), `-l` (a flag on macOS), and
        // `arch`, which runs a program on macOS.
        "p='1w x'; sed -n \"$p\" notes.txt",
        "sed -n '${p}' notes.txt",
        "bash -c 'sed -n \"$1\" notes.txt' _ '1w x'",
        "sed '1w x' -e p notes.txt",
        "sed -l '1w x' p notes.txt",
        // Review r2: an operand or script a shell expands, or a glob.
        "f=-e1w.githooks/pre-commit; sed -n 1p \"$f\" notes.txt",
        "bash -c 'sed -n 1p \"$1\" notes.txt' _ -e1wx",
        "sed -n \"$@\" notes.txt",
        "sed -n 1p *.txt",
        // Review r3: zsh's `$=p` splits `$p`; `$=` is refused altogether.
        "p=1w.githooks/pre-commit; sed -n -e \"$=p\" notes.txt",
        "sed -n -e \"$==p\" notes.txt",
        "zsh -c 'sed -n -e \"$=1\" notes.txt' _ 1wx",
        "sed -ne '$=' -e 2d notes.txt",
        "sed -n '$p' notes.txt",
        "sed -n -e =nl notes.txt",
        "arch -arm64 sh -c 'echo x > .githooks/pre-commit'",
        "bash script.sh",
        "python3 -c 'open(\"x\", \"w\")'",
        "ln -s ../x",
        "(printf x > notes.txt)",
        // The destination comes first, or a variable moves the config file.
        "cp -t .husky pre-commit",
        "install --target-directory=.husky pre-commit",
        "HOME=. git config --global a.b c",
        "GIT_CONFIG_GLOBAL=x git config --global a.b c",
        "NPM_CONFIG_USERCONFIG=x npm config set a b",
        // #1182: curl options that write or may (output files, headers,
        // cookie jars, a config file, an upload to `file://`, a `-w` that
        // writes or comes from a file, traces), `--name=value`, `--`, or
        // ones it does not know; `command -p` runs its program.
        "curl -o out.html https://x",
        "curl -s -o /dev/null https://x",
        "curl -sSo out https://x",
        "curl --output out https://x",
        "curl -O https://x/f",
        "curl -sSO https://x/f",
        "curl --remote-name https://x/f",
        "curl --create-dirs https://x",
        "curl -D headers.txt https://x",
        "curl -c jar https://x",
        "curl -K cfg https://x",
        "curl -T f file:///tmp/x",
        "curl --stderr log https://x",
        "curl --libcurl code.c https://x",
        "curl --output-dir d https://x",
        "curl -w '%output{x}%{http_code}' https://x",
        "curl -w @fmt https://x",
        "curl --trace t https://x",
        "curl --max-time=5 https://x",
        "curl --no-silent https://x",
        "curl -H",
        "curl -- -o x",
        "command -p touch notes.txt",
        "command -x foo",
    ] {
        assert!(!of(script).understood, "{script}: {:?}", of(script));
    }
    // #1182: not through a shell, a `-w` format is read as curl reads it.
    let argv = |words: &[&str]| words.iter().map(ToString::to_string).collect::<Vec<_>>();
    assert_eq!(
        writes(&argv(&["curl", "-sS", "-w", "%{http_code}", "https://x"])),
        known(&[])
    );
    assert!(!writes(&argv(&["curl", "-w", "%output{a}", "https://x"])).understood);
    // Not through a shell, `>` in a value is text.
    assert_eq!(
        writes(&[
            "git".to_string(),
            "config".to_string(),
            "--global".to_string(),
            "user.name".to_string(),
            "x > y".to_string(),
        ]),
        known(&["~/.gitconfig"])
    );
}

/// #1182: [`writes`] never follows a `cd`; [`writes_following_cd`] spells
/// the writes after `cd DIR &&` from `DIR`, and only when the folder is
/// certain.
#[test]
fn issue_1182_writes_following_cd() {
    for script in ["cd /p && echo x > y", "cd .husky && printf x > pre-commit"] {
        assert!(!writes(&shell(script)).understood, "{script}");
    }
    let of = |script: &str| writes_following_cd(&shell(script));
    for (script, expected) in [
        (
            "cd /p/project && echo x >> ../outside-notes.txt",
            &["/p/project/../outside-notes.txt"][..],
        ),
        (
            "cd sub && printf 'x\\n' >> ../outside-notes.txt && cat ../outside-notes.txt",
            &["sub/../outside-notes.txt"],
        ),
        ("cd .husky && printf x > pre-commit", &[".husky/pre-commit"]),
        // #1212: quotes without a separator in them.
        ("cd sub && printf 'a b' > \"c d\"", &["sub/c d"]),
        ("cd a && cd b && touch x ~/y", &["a/b/x", "~/y"]),
        ("cd -P sub && echo x | tee out", &["sub/out"]),
        ("cd && echo x > notes", &["~/notes"]),
        ("cd ~/w && echo x > notes", &["~/w/notes"]),
        ("cd /tmp 2>/dev/null && ls", &["/dev/null"]),
        ("echo x > a && cd sub", &["a"]),
        ("echo x > a; cat b", &["a"]),
    ] {
        assert_eq!(of(script), known(expected), "{script}");
    }
    // Where the rest runs depends on whether `cd` worked, or it runs in a
    // subshell, or its folder is one a shell expands, or not known.
    for script in [
        "cd x; echo y > z",
        "cd x || exit; echo y > z",
        "cd x && echo a; echo b > c",
        "cd x && echo a || echo b > c",
        "cd x && sleep 1 & echo b > c",
        "true || cd .husky && echo x > .codex/x",
        "true | cd .husky && echo x > .codex/x",
        "echo a; cd x && echo b > c",
        "cd x | tee y",
        "cd sub && cd - && echo x > .codex/config.toml",
        "cd \"$PWD\" && echo x > .codex/config.toml",
        "cd $HOME && echo x > y",
        "cd ~other && echo x > y",
        "cd '' && echo x > y",
        "cd * && echo y > z",
        "cd a b && echo y > z",
        "X=1 cd sub && echo y > z",
        "cd x && pushd y && echo z > w",
        "bash -c 'cd sub && echo x > y'",
        // #1212: a quoted or escaped separator; zsh's folder stack.
        "echo '&&' cd x && printf y > .husky/pre-commit",
        "echo \\&\\& cd x && printf y > z",
        "echo 'a;' cd x && printf y > z",
        "cd x && echo \"|\" tee ../y",
        "cd +0 && echo y > z",
        "cd -P +1 && echo y > z",
    ] {
        assert!(!of(script).understood, "{script}: {:?}", of(script));
    }
}

/// #736: the macOS keychain commands, also inside a script; not other
/// `security` commands or words.
#[test]
fn issue_736_keychain_commands() {
    for command in [
        shell("security find-generic-password -s probe"),
        shell("/usr/bin/security find-internet-password -s x -w"),
        shell("FOO=1 sudo security dump-keychain; echo done"),
        shell("sudo -u root security -q export -k x"),
        shell("echo start && security find-certificate -a"),
        shell("bash -c 'security add-generic-password -s x -w y'"),
        vec![
            "security".to_string(),
            "unlock-keychain".to_string(),
            "-p".to_string(),
            "x".to_string(),
        ],
    ] {
        assert!(keychain_command(&command), "{command:?}");
    }
    for command in [
        shell("security list-keychains"),
        shell("security verify-cert -c cert.pem"),
        shell("echo find-generic-password"),
        shell("cat security.md"),
        // `security` as a folder, `export` as the shell builtin.
        shell("ls core/src/security | grep export"),
        shell("cd codex-rs/core/src/security && export RUST_LOG=debug && cargo test"),
        shell("security list-keychains; export X=1"),
        shell("git commit -m \"fix security export\""),
        shell("echo security find-generic-password"),
    ] {
        assert!(!keychain_command(&command), "{command:?}");
    }
}

/// #826 follow-up: commands that run a program needing a user namespace,
/// also inside a script, found as keychain commands are; not the words
/// elsewhere.
#[test]
fn issue_826_user_namespace_commands() {
    use super::writes::user_namespace_command;
    for command in [
        shell("unshare -Ur true"),
        shell("/usr/bin/bwrap --ro-bind / / true"),
        shell("cd app && podman run --rm alpine true"),
        shell("FOO=1 buildah bud ."),
        shell("bash -c 'flatpak run org.example.App'"),
        vec!["unshare".to_string(), "-Urm".to_string(), "sh".to_string()],
    ] {
        assert!(user_namespace_command(&command), "{command:?}");
    }
    for command in [
        shell("echo unshare"),
        shell("cat bwrap.md"),
        shell("docker run --rm alpine true"),
        shell("git commit -m \"use podman\""),
        shell("ls podman/"),
    ] {
        assert!(!user_namespace_command(&command), "{command:?}");
    }
}

/// #740 (and #737): under Aggressive's sandbox (the current folder only),
/// which commands a grant could never help, and why; a hook in a folder
/// `core.hooksPath` names keeps the grant, and so does a start-up file when
/// the current folder is the home folder.
#[cfg(unix)]
#[test]
fn issue_740_unliftable_notes() {
    use codex_protocol::permissions::NetworkSandboxPolicy;
    let base = tempfile::tempdir().unwrap();
    let base = base.path().canonicalize().unwrap();
    let home = base.join("home");
    let work = home.join("work");
    for folder in [work.join(".git/hooks"), work.join(".husky")] {
        std::fs::create_dir_all(&folder).unwrap();
    }
    std::fs::write(work.join(".git/config"), "[core]\n\thooksPath = .husky\n").unwrap();
    let inputs = |folder: &Path| {
        let folder = AbsolutePathBuf::from_absolute_path(folder).unwrap();
        let user_home = if folder.as_path() == work.as_path() {
            home.clone()
        } else {
            folder.to_path_buf()
        };
        ReadPolicyInputs {
            codex_home: base.join("corbanu-home"),
            sqlite_home: base.join("corbanu-home"),
            turn_cwd: folder.clone(),
            workspace_roots: vec![folder],
            profile: PermissionProfile::workspace_write_with(
                &[],
                NetworkSandboxPolicy::Restricted,
                /*exclude_tmpdir_env_var*/ true,
                /*exclude_slash_tmp*/ true,
            ),
            user_home: Some(user_home),
            keychain_closed: false,
        }
    };
    let in_work = inputs(&work);
    let note = |script: &str| {
        in_work
            .unliftable_note(&shell(script), &work)
            .map(|note| note.explanation())
    };
    assert_eq!(
        note("echo t04 >> ../outside-notes.txt"),
        Some(outside_folder_explanation("../outside-notes.txt"))
    );
    assert_eq!(
        note("npm config set registry https://x"),
        Some(outside_folder_explanation("~/.npmrc"))
    );
    assert_eq!(
        note("echo x > /tmp/x.txt 2>/dev/null"),
        Some(outside_folder_explanation("/tmp/x.txt"))
    );
    assert_eq!(
        note("printf '#!/bin/sh\\n' > .git/hooks/pre-commit"),
        Some(read_only_in_folder_explanation(".git/hooks/pre-commit"))
    );
    assert_eq!(
        note("git config core.hooksPath /tmp"),
        Some(read_only_in_folder_explanation(".git/config"))
    );
    // #1182: a write outside the folder after `cd` (AGG-20 F06, F18), and
    // a curl writing outside it.
    let work_shown = work.display();
    assert_eq!(
        note(&format!(
            "cd {work_shown} && echo x >> ../outside-notes.txt"
        )),
        Some(outside_folder_explanation(&format!(
            "{work_shown}/../outside-notes.txt"
        )))
    );
    assert_eq!(
        note("cd .husky && printf 'x\\n' >> ../../outside-notes.txt"),
        Some(outside_folder_explanation(".husky/../../outside-notes.txt"))
    );
    assert_eq!(
        note("curl -sS https://example.com > /tmp/probe1.out"),
        Some(outside_folder_explanation("/tmp/probe1.out"))
    );
    for script in [
        "ln -s notes.txt notes-link.txt && cat notes-link.txt",
        // #789: the commands of AGG-R14's child agent and parent.
        "sleep 60; echo finished > ks-child.txt",
        "sleep 60",
        "sed -n '1,240p' notes.txt",
        "cat .codex/notes.md",
        "echo x > notes.txt 2>/dev/null",
        "git config --global --get user.name",
        // #1182: AGG-20's network probe and lookup, and a write inside the
        // folder after `cd`.
        "curl -sS -I https://example.com",
        "command -v corbanu",
        "cd .husky && cat pre-commit > ../notes.txt",
    ] {
        assert_eq!(note(script), Some(NOTHING_TO_LIFT.to_string()), "{script}");
    }
    // A grant lifts it, or may: offered.
    for script in [
        "./missing-script.sh | tee ../install.log",
        "npx husky install > ../install.log",
        "printf x > .husky/pre-commit",
        "printf x > .husky/pre-commit; echo x >> ../outside.txt",
        "./install-hooks.sh",
        "printf x > $DIR/pre-commit",
        "echo x > notes.txt && npm install",
        // #1182: a grant lifts it; a `cd` not followed; curl writing a file.
        "cd .husky && printf x > pre-commit",
        "cd sub; printf x > .husky/pre-commit",
        "cd \"$PWD\" && echo x > .codex/config.toml",
        "curl -s -o /dev/null https://example.com",
        // #1212: the `&&` before `cd` is quoted; the hook is `./.husky`'s.
        "echo '&&' cd x && printf y > .husky/pre-commit",
    ] {
        assert_eq!(note(script), None, "{script}");
    }
    // The current folder is the home folder: start-up files are liftable
    // (on Linux only existing ones are protected at all). The protected
    // paths are the real home folder's.
    if cfg!(target_os = "macos") {
        let home = dirs::home_dir().unwrap().canonicalize().unwrap();
        let in_home = inputs(&home);
        for script in ["echo x >> ~/.zshrc", "git config --global a.b c"] {
            assert!(
                in_home.unliftable_note(&shell(script), &home).is_none(),
                "{script}"
            );
        }
    }
    // Where a command failed: outside, or in the project's `.git`.
    assert_eq!(in_work.unliftable_write(&base.join("x")), Some(false));
    assert_eq!(
        in_work.unliftable_write(&work.join(".git/hooks/pre-commit")),
        Some(true)
    );
    assert_eq!(in_work.unliftable_write(&work.join("notes.txt")), None);
    assert_eq!(in_work.unliftable_write(Path::new("/dev/null")), None);
}

/// #736: under Aggressive (macOS), a keychain command is explained before
/// it runs (no grant) and where it failed, and listed under the
/// inspector's recent denials; not under Permissive.
#[cfg(target_os = "macos")]
#[tokio::test]
async fn issue_736_keychain_command_is_explained_under_aggressive() {
    let (session, turn) = crate::session::tests::make_session_and_context().await;
    #[allow(deprecated)]
    let cwd = turn.cwd.clone();
    let command = shell("security find-generic-password -s probe");
    let output = "security: SecKeychainSearchCopyNext: The specified item could not be found \
                  in the keychain.\n";
    let explain = async || {
        denial_explanation(&session, &turn, &command, cwd.as_path(), Some(44), output).await
    };
    assert_eq!(explain().await, None, "Permissive");
    let client = (*session.services.model_client())
        .clone()
        .with_ingress_level(SecurityLevel::Aggressive)
        .with_source_envelopes(/*enabled*/ true);
    session.services.replace_model_client(client);
    assert_eq!(explain().await, Some(KEYCHAIN_DENIAL.to_string()));
    assert_eq!(
        denial_explanation(&session, &turn, &command, cwd.as_path(), Some(0), "x").await,
        None,
        "it succeeded"
    );
    let note = protected_note_in_command(&session, &turn, &command, cwd.as_path())
        .await
        .map(|note| note.explanation());
    assert_eq!(note, Some(KEYCHAIN_PROMPT.to_string()));
    let denials: Vec<_> = crate::security::inspection::observe(
        turn.config.codex_home.as_path(),
        SecurityLevel::Aggressive,
        Some(session.thread_id()),
        crate::security::aggressive::now_unix_seconds(),
    )
    .denials
    .into_iter()
    .filter(|denial| denial.thread == Some(session.thread_id()))
    .map(|denial| denial.reason)
    .collect();
    assert_eq!(denials, vec!["using the macOS keychain"]);
}
