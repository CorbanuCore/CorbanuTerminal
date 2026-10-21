use super::*;
use codex_security_policy::SecurityLevel;
use pretty_assertions::assert_eq;

fn shell(script: &str) -> Vec<String> {
    vec!["/bin/zsh".into(), "-lc".into(), script.into()]
}

/// The word the command used for the first denied path it names, through
/// quoting, redirections, `--flag=` values and home variables.
#[test]
fn issue_442_finds_the_denied_path_the_command_names() {
    let cwd = Path::new("/work/project");
    let user_home = Path::new("/Users/someone");
    let codex_home = Path::new("/Users/someone/.corbanu");
    let denied = |path: &Path| {
        [
            Path::new("/work/project/.env"),
            Path::new("/Users/someone/.ssh"),
            Path::new("/Users/someone/.corbanu/auth.json"),
        ]
        .iter()
        .any(|denied| path.starts_with(denied))
    };
    let word = |script: &str| {
        denied_word(
            &shell(script),
            cwd,
            Some(user_home),
            codex_home,
            denied,
            /*denial_lines*/ None,
        )
    };
    assert_eq!(word("cat .env"), Some(".env".to_string()));
    assert_eq!(word("head -3 '.env' | sort"), Some(".env".to_string()));
    assert_eq!(word("wc -l <.env;"), Some(".env".to_string()));
    assert_eq!(word("tool --file=.env"), Some(".env".to_string()));
    assert_eq!(word("cat ~/.ssh/config"), Some("~/.ssh/config".to_string()));
    assert_eq!(
        word("cat $HOME/.ssh/id_ed25519"),
        Some("$HOME/.ssh/id_ed25519".to_string())
    );
    assert_eq!(
        word("cat \"$CORBANU_HOME/auth.json\""),
        Some("$CORBANU_HOME/auth.json".to_string())
    );
    assert_eq!(word("cat notes.txt"), None);
    assert_eq!(word("cat $UNKNOWN/.env.example"), None);
    // Tied to the denial the output reports: an unrelated failure in a
    // command that also names `.env` is not attributed to it.
    let lines = ["grep: ./other.txt: Permission denied".to_string()];
    assert_eq!(
        denied_word(
            &shell("grep -r x . --exclude=.env"),
            cwd,
            Some(user_home),
            codex_home,
            denied,
            Some(&lines),
        ),
        None
    );
    let lines = ["cat: .env: Operation not permitted".to_string()];
    assert_eq!(
        denied_word(
            &shell("cat .env"),
            cwd,
            Some(user_home),
            codex_home,
            denied,
            Some(&lines)
        ),
        Some(".env".to_string())
    );
}

/// #506 review: a denial line names the folder as the shell resolved it, so
/// `ls "$CODEX_HOME"` and `ls home/tmp/..` are attributed to the home; the
/// listing explanation names the path as the command spelled it.
#[test]
#[cfg_attr(
    windows,
    ignore = "Unix fixtures (POSIX paths and shell syntax); Windows coverage is #691"
)]
fn issue_506_listing_denial_is_attributed_through_variables_and_dot_dot() {
    let cwd = Path::new("/work/project");
    let user_home = Path::new("/users/me");
    let codex_home = Path::new("/users/me/.corbanu");
    let denied = |path: &Path| {
        AbsolutePathBuf::from_absolute_path(path).is_ok_and(|path| path.as_path() == codex_home)
    };
    let lines = ["ls: /users/me/.corbanu: Operation not permitted".to_string()];
    for (script, expected) in [
        ("ls -la \"$CODEX_HOME\"", "$CODEX_HOME"),
        ("ls ${CORBANU_HOME}", "${CORBANU_HOME}"),
        ("ls ~/.corbanu/tmp/..", "~/.corbanu/tmp/.."),
    ] {
        assert_eq!(
            denied_word(
                &shell(script),
                cwd,
                Some(user_home),
                codex_home,
                denied,
                Some(&lines),
            ),
            Some(expected.to_string()),
            "{script}"
        );
    }
    let other = ["ls: /users/me/other: Operation not permitted".to_string()];
    assert_eq!(
        denied_word(
            &shell("ls \"$CODEX_HOME\" other"),
            cwd,
            Some(user_home),
            codex_home,
            denied,
            Some(&other),
        ),
        None
    );
    let text = listing_explanation("$CODEX_HOME");
    assert!(
        text.starts_with("Security level Aggressive blocked listing `$CODEX_HOME`"),
        "{text}"
    );
    assert!(text.contains("not even once"), "{text}");
}

/// #554: `find ~` or `ls -aR ~` reports the home's listing denial below the
/// folder the command names; the line is attributed to the home as it
/// spells it, in BSD and GNU forms and relative to the folder a `cd` names.
#[test]
#[cfg_attr(
    windows,
    ignore = "Unix fixtures (POSIX paths and shell syntax); Windows coverage is #691"
)]
fn issue_554_recursive_listing_of_a_parent_is_attributed_to_the_home() {
    let cwd = Path::new("/work/project");
    let user_home = Path::new("/users/me");
    let codex_home = Path::new("/users/me/.corbanu");
    let unlistable = |path: &Path| {
        AbsolutePathBuf::from_absolute_path(path).is_ok_and(|path| path.as_path() == codex_home)
    };
    let found = |script: &str, line: &str| {
        unlistable_path_in_lines(
            &shell(script),
            cwd,
            Some(user_home),
            codex_home,
            &[line.to_string()],
            &unlistable,
        )
    };
    for (script, line, expected) in [
        (
            "find ~ -name '*.toml'",
            "find: /users/me/.corbanu: Operation not permitted",
            "/users/me/.corbanu",
        ),
        (
            "ls -aR \"$HOME\"",
            "ls: /users/me/.corbanu: Operation not permitted",
            "/users/me/.corbanu",
        ),
        (
            "find /users/me",
            "find: \u{2018}/users/me/.corbanu\u{2019}: Permission denied",
            "/users/me/.corbanu",
        ),
        (
            "ls -aR /users/me",
            "ls: cannot open directory '/users/me/.corbanu': Permission denied",
            "/users/me/.corbanu",
        ),
        (
            "cd ~ && find .",
            "find: ./.corbanu: Operation not permitted",
            "./.corbanu",
        ),
        (
            "cd ~; find .",
            "find: ./.corbanu: Operation not permitted",
            "./.corbanu",
        ),
        (
            "ls -aR ../../users/me",
            "ls: ../../users/me/.corbanu: Operation not permitted",
            "../../users/me/.corbanu",
        ),
        // #1194: the program's name was split off (written separately, with
        // the command's output merged in between).
        (
            "find ~",
            "/users/me/.corbanu: Operation not permitted",
            "/users/me/.corbanu",
        ),
        (
            "cd ~ && ls -aR .",
            "./.corbanu: Operation not permitted",
            "./.corbanu",
        ),
    ] {
        assert_eq!(found(script, line), Some(expected.to_string()), "{script}");
    }
    for (script, line) in [
        // Another folder, or a name that only starts like the home's.
        (
            "find ~",
            "find: /users/me/Library/Mail: Operation not permitted",
        ),
        (
            "find ~",
            "find: /users/me/.corbanu-old: Operation not permitted",
        ),
        // A path below the home, not the home itself.
        (
            "find ~",
            "find: /users/me/.corbanu/vault: Operation not permitted",
        ),
        // Relative to a folder the command does not name.
        ("find .", "find: ./.corbanu: Operation not permitted"),
        // #1194: with the program's name split off, still only the home.
        (
            "find ~",
            "/users/me/.corbanu/vault: Operation not permitted",
        ),
        ("find ~", "/users/me/.corbanu-old: Operation not permitted"),
    ] {
        assert_eq!(found(script, line), None, "{script}: {line}");
    }
}

/// #442 (milestone AGG-09): under Aggressive a denied read of a protected
/// file is explained where it failed, naming the level and /security; it is
/// not explained under Permissive, for a command that succeeded, or for a
/// failure that is not a denial.
#[tokio::test]
async fn issue_442_denied_protected_read_is_explained_under_aggressive() {
    let (session, turn) = crate::session::tests::make_session_and_context().await;
    #[allow(deprecated)]
    let cwd = turn.cwd.clone();
    let home = turn.config.codex_home.to_path_buf();
    std::fs::create_dir_all(&home).unwrap();
    std::fs::write(home.join("auth.json"), "x").unwrap();
    let command = shell(&format!("cat '{}'", home.join("auth.json").display()));
    let denial = "cat: auth.json: Operation not permitted\n";
    let explain = async |exit_code, output: &str| {
        denial_explanation(
            &session,
            &turn,
            &command,
            cwd.as_path(),
            Some(exit_code),
            output,
        )
        .await
    };
    assert_eq!(explain(1, denial).await, None, "Permissive: unchanged");

    let client = (*session.services.model_client())
        .clone()
        .with_ingress_level(SecurityLevel::Aggressive)
        .with_source_envelopes(/*enabled*/ true);
    session.services.replace_model_client(client);
    let explanation = explain(1, denial)
        .await
        .expect("explained under Aggressive");
    assert!(
        explanation.starts_with("Security level Aggressive blocked access to"),
        "{explanation}"
    );
    assert!(
        explanation.contains("cannot be granted, not even once"),
        "{explanation}"
    );
    assert!(
        explanation.contains("switch to Permissive in /security"),
        "{explanation}"
    );
    assert_eq!(explain(0, denial).await, None);
    assert_eq!(
        explain(1, "cat: auth.json: No such file or directory\n").await,
        None
    );
    assert_eq!(
        protected_path_in_command(&session, &turn, &command, cwd.as_path()).await,
        Some(home.join("auth.json").display().to_string())
    );
}

/// #554: under Aggressive, `find` of the folder above the Corbanu home is
/// explained like a direct listing of the home (#506); not under Permissive,
/// and not for a denial of another folder.
#[tokio::test]
async fn issue_554_find_above_the_home_is_explained_under_aggressive() {
    let (session, turn) = crate::session::tests::make_session_and_context().await;
    #[allow(deprecated)]
    let cwd = turn.cwd.clone();
    let home = turn.config.codex_home.to_path_buf();
    std::fs::create_dir_all(&home).unwrap();
    let parent = home.parent().expect("the home has a parent").to_path_buf();
    let command = shell(&format!("find '{}' -name '*.toml'", parent.display()));
    let denial = format!("find: {}: Operation not permitted\n", home.display());
    let explain = async |output: &str| {
        denial_explanation(&session, &turn, &command, cwd.as_path(), Some(1), output).await
    };
    assert_eq!(explain(&denial).await, None, "Permissive: unchanged");

    let client = (*session.services.model_client())
        .clone()
        .with_ingress_level(SecurityLevel::Aggressive)
        .with_source_envelopes(/*enabled*/ true);
    session.services.replace_model_client(client);
    let explanation = explain(&denial).await.expect("explained under Aggressive");
    assert_eq!(
        explanation,
        listing_explanation(&home.display().to_string())
    );
    let other = format!(
        "find: {}: Operation not permitted\n",
        parent.join("elsewhere").display()
    );
    assert_eq!(explain(&other).await, None);
}

/// #1194: BSD `find` and `ls` write `find: ` and the rest of the denial line
/// separately, and unified exec merges stdout and stderr as they arrive, so
/// the listing they flush at exit can land in between. The denial is still
/// explained as #554's.
#[tokio::test]
async fn issue_1194_a_split_denial_line_is_still_explained() {
    let (session, turn) = crate::session::tests::make_session_and_context().await;
    #[allow(deprecated)]
    let cwd = turn.cwd.clone();
    let home = turn.config.codex_home.to_path_buf();
    std::fs::create_dir_all(&home).unwrap();
    let parent = home.parent().expect("the home has a parent").to_path_buf();
    let client = (*session.services.model_client())
        .clone()
        .with_ingress_level(SecurityLevel::Aggressive)
        .with_source_envelopes(/*enabled*/ true);
    session.services.replace_model_client(client);
    let (parent, home) = (parent.display(), home.display());
    for (command, program, listing) in [
        (
            format!("find '{parent}'"),
            "find",
            format!("{parent}\n{home}\n"),
        ),
        (
            format!("ls -aR '{parent}'"),
            "ls",
            format!("{parent}:\n.\n..\n{home}\n\n"),
        ),
    ] {
        let command = shell(&command);
        let output = format!("{program}: {listing}{home}: Operation not permitted\n");
        let explanation =
            denial_explanation(&session, &turn, &command, cwd.as_path(), Some(1), &output).await;
        assert_eq!(
            explanation,
            Some(listing_explanation(&home.to_string())),
            "{output}"
        );
    }
}

/// #621 (milestone AGG-R07): a path inside code an interpreter runs is found
/// like a path argument, so no grant is offered for it; code that names no
/// protected path is left alone.
#[test]
#[cfg_attr(
    windows,
    ignore = "Unix fixtures (POSIX paths and shell syntax); Windows coverage is #691"
)]
fn issue_621_finds_the_denied_path_inside_interpreter_code() {
    let cwd = Path::new("/work/project");
    let user_home = Path::new("/Users/someone");
    let codex_home = Path::new("/Users/someone/.corbanu");
    let denied = |path: &Path| {
        AbsolutePathBuf::from_absolute_path(path).is_ok_and(|path| {
            path.as_path().starts_with(codex_home)
                || path.as_path() == Path::new("/work/project/.env")
        })
    };
    let word = |script: &str| {
        denied_word(
            &shell(script),
            cwd,
            Some(user_home),
            codex_home,
            denied,
            /*denial_lines*/ None,
        )
    };
    for (script, expected) in [
        (
            "python3 -c \"import os; print(os.listdir('/Users/someone/.corbanu'))\"",
            "/Users/someone/.corbanu",
        ),
        (
            "python -c 'print(open(\"/Users/someone/.corbanu/auth.json\").read())'",
            "/Users/someone/.corbanu/auth.json",
        ),
        (
            "node -e \"console.log(require('fs').readdirSync('$CORBANU_HOME'))\"",
            "$CORBANU_HOME",
        ),
        (
            "perl -e 'opendir(D, \"~/.corbanu\") or die; print readdir(D)'",
            "~/.corbanu",
        ),
        (
            "ruby -e 'puts Dir.entries(\"/Users/someone/.corbanu\")'",
            "/Users/someone/.corbanu",
        ),
        ("python3 -c \"print(open('.env').read())\"", ".env"),
    ] {
        assert_eq!(word(script), Some(expected.to_string()), "{script}");
    }
    for script in [
        "python3 -c \"print(1 + 2)\"",
        "python3 -c \"import os; print(os.listdir('/work/project'))\"",
        "node -e \"console.log('hello, world')\"",
    ] {
        assert_eq!(word(script), None, "{script}");
    }
}

/// An Aggressive session (with the source labels it turns on).
#[cfg(unix)]
fn aggressive(session: &Session) {
    let client = (*session.services.model_client())
        .clone()
        .with_ingress_level(SecurityLevel::Aggressive)
        .with_source_envelopes(/*enabled*/ true);
    session.services.replace_model_client(client);
}

/// The recent denials the inspector shows for `session` (not the
/// process-wide ones other tests in this process may have recorded).
#[cfg(unix)]
fn recent_denials(session: &Session, turn: &TurnContext) -> Vec<(&'static str, &'static str)> {
    crate::security::inspection::observe(
        turn.config.codex_home.as_path(),
        SecurityLevel::Aggressive,
        Some(session.thread_id()),
        crate::security::aggressive::now_unix_seconds(),
    )
    .denials
    .into_iter()
    .filter(|denial| denial.thread == Some(session.thread_id()))
    .map(|denial| (denial.reason, denial.outcome))
    .collect()
}

/// #621 and #622: under Aggressive, a protected read or home listing an
/// interpreter makes is explained where it failed, also when the command
/// built the path at run time (the denial line names it), and it is listed
/// under the inspector's recent denials. The approval prompt for the listing
/// offers no grant (`protected_note_in_command`; Windows does not enforce
/// the listing denial, so it says nothing there).
#[cfg(unix)]
#[tokio::test]
async fn issue_621_622_interpreter_denials_are_explained_and_listed() {
    let (session, turn) = crate::session::tests::make_session_and_context().await;
    #[allow(deprecated)]
    let cwd = turn.cwd.clone();
    let home = turn.config.codex_home.to_path_buf();
    std::fs::create_dir_all(&home).unwrap();
    std::fs::write(home.join("auth.json"), "x").unwrap();
    let listing = shell(&format!(
        "python3 -c \"import os; print(os.listdir('{}'))\"",
        home.display()
    ));
    let listing_denied = format!(
        "Traceback (most recent call last):\n  File \"<string>\", line 1, in <module>\n\
         PermissionError: [Errno 1] Operation not permitted: '{}'\n",
        home.display()
    );
    // The path is built at run time: no word names it.
    let built = shell("python3 -c \"import os; open(os.environ['P'] + '/auth.json').read()\"");
    let built_denied = format!(
        "PermissionError: [Errno 1] Operation not permitted: '{}'\n",
        home.join("auth.json").display()
    );
    let explain = async |command: &[String], output: &str| {
        denial_explanation(&session, &turn, command, cwd.as_path(), Some(1), output).await
    };
    assert_eq!(explain(&listing, &listing_denied).await, None, "Permissive");
    assert_eq!(recent_denials(&session, &turn), Vec::new());

    aggressive(&session);
    assert_eq!(
        explain(&listing, &listing_denied).await,
        Some(listing_explanation(&home.display().to_string()))
    );
    assert_eq!(
        explain(&built, &built_denied).await,
        Some(explanation(&home.join("auth.json").display().to_string()))
    );
    let note = protected_note_in_command(&session, &turn, &listing, cwd.as_path())
        .await
        .expect("the listing is recognised before it runs");
    assert!(
        note.explanation()
            .contains("cannot be granted, not even once"),
        "{}",
        note.explanation()
    );
    let outcome = "denied by the sandbox; never granted under Aggressive";
    assert_eq!(
        recent_denials(&session, &turn),
        vec![
            ("reading a protected file", outcome),
            ("listing the Corbanu home", outcome),
        ]
    );
    // A failure that is not a denial of a protected path is not listed.
    assert_eq!(
        explain(
            &built,
            "PermissionError: [Errno 13] Permission denied: '/elsewhere/file'\n"
        )
        .await,
        None
    );
    assert_eq!(recent_denials(&session, &turn).len(), 2);
}

/// #620 (milestone AGG-18, AGG-R17): under Aggressive a start-up file
/// outside the current folder (`~/.zshrc`) can never be written, not even
/// with Grant once, so the approval prompt offers no grant and says why;
/// a denied write is explained the same way where it failed. A write in the
/// current folder, and Permissive sessions, are left alone.
#[cfg(unix)]
#[tokio::test]
async fn issue_620_startup_file_outside_the_folder_is_never_granted() {
    let (session, turn) = crate::session::tests::make_session_and_context().await;
    #[allow(deprecated)]
    let cwd = turn.cwd.clone();
    let user_home = dirs::home_dir().expect("a home folder");
    let append = shell("echo '# grant-once-1' >> ~/.zshrc");
    let denied = format!(
        "zsh:1: operation not permitted: {}\n",
        user_home.join(".zshrc").display()
    );
    let note = async |command: &[String]| {
        protected_note_in_command(&session, &turn, command, cwd.as_path())
            .await
            .map(|note| note.explanation())
    };
    assert_eq!(note(&append).await, None, "Permissive");

    aggressive(&session);
    let explained = note(&append).await.expect("no grant for ~/.zshrc");
    assert_eq!(explained, startup_explanation("~/.zshrc"));
    assert!(
        explained.contains("cannot be granted, not even once"),
        "{explained}"
    );
    assert!(explained.contains("Permissive in /security"), "{explained}");
    assert_eq!(
        note(&shell("echo '# x' >> \"$HOME/.bash_profile\"")).await,
        Some(startup_explanation("$HOME/.bash_profile"))
    );
    assert_eq!(
        note(&shell(
            "cp notes.txt ~/.bashrc && tee -a ~/.zshenv < notes.txt"
        ))
        .await,
        Some(startup_explanation("~/.bashrc"))
    );
    assert_eq!(
        note(&shell("printf x>>~/.zshrc")).await,
        Some(startup_explanation("~/.zshrc"))
    );
    // Only where the command writes it: reading one, running a program from
    // `~/.local/bin` or naming one in a message is not a start-up file write
    // (#740 explains a write no grant lifts instead; an unknown program
    // keeps the grant).
    for script in [
        "~/.local/bin/pre-commit install",
        "git commit -m 'document ~/.zshrc > notes'",
    ] {
        assert_eq!(note(&shell(script)).await, None, "{script}");
    }
    // This session's profile may not write its folder at all: then a write
    // there is outside the writable folders, else read-only or plain.
    for (script, word) in [
        ("cat ~/.gitconfig >> .git/config", ".git/config"),
        (
            "grep x ~/.bashrc > .git/hooks/pre-commit 2>&1",
            ".git/hooks/pre-commit",
        ),
        ("echo '# x' >> notes.txt", "notes.txt"),
    ] {
        let explained = note(&shell(script)).await;
        assert!(
            [
                outside_folder_explanation(word),
                read_only_in_folder_explanation(word),
                NOTHING_TO_LIFT.to_string(),
            ]
            .iter()
            .any(|expected| explained.as_ref() == Some(expected))
                && (word == "notes.txt" || explained != Some(NOTHING_TO_LIFT.to_string())),
            "{script}: {explained:?}"
        );
    }

    assert_eq!(
        denial_explanation(&session, &turn, &append, cwd.as_path(), Some(1), &denied).await,
        Some(startup_explanation("~/.zshrc"))
    );
    assert_eq!(
        recent_denials(&session, &turn),
        vec![(
            "writing a start-up file outside the current folder",
            "denied by the sandbox; never granted under Aggressive"
        )]
    );
}

/// #620: a start-up file reached through links counts by every name on the
/// way (`./rc -> ~/.zshrc -> ~/dotfiles/zshrc`, as dotfile managers link
/// it), and is writable where any of those names is (a workspace that is
/// the home folder keeps Grant once).
#[cfg(unix)]
#[test]
fn issue_620_start_up_files_through_links() {
    use codex_protocol::permissions::NetworkSandboxPolicy;
    let user_home = tempfile::tempdir().unwrap();
    let work = tempfile::tempdir().unwrap();
    let home = user_home.path();
    std::fs::create_dir(home.join("dotfiles")).unwrap();
    std::fs::write(home.join("dotfiles/zshrc"), "").unwrap();
    std::os::unix::fs::symlink(home.join("dotfiles/zshrc"), home.join(".zshrc")).unwrap();
    std::os::unix::fs::symlink(home.join(".zshrc"), work.path().join("rc")).unwrap();
    let inputs = |folder: &Path| {
        let folder = AbsolutePathBuf::from_absolute_path(folder.canonicalize().unwrap()).unwrap();
        ReadPolicyInputs {
            codex_home: home.join(".corbanu"),
            sqlite_home: home.join(".corbanu"),
            turn_cwd: folder.clone(),
            workspace_roots: vec![folder],
            // Aggressive's sandbox: the current folder only.
            profile: PermissionProfile::workspace_write_with(
                &[],
                NetworkSandboxPolicy::Restricted,
                /*exclude_tmpdir_env_var*/ true,
                /*exclude_slash_tmp*/ true,
            ),
            user_home: Some(home.to_path_buf()),
            keychain_closed: false,
        }
    };
    let in_work = inputs(work.path());
    assert!(in_work.unwritable_startup_file(&work.path().join("rc")));
    assert!(in_work.unwritable_startup_file(&home.join(".zshrc")));
    assert!(in_work.unwritable_startup_file(&home.join(".bash_profile")));
    assert!(!in_work.unwritable_startup_file(&work.path().join("notes.txt")));
    assert!(!in_work.unwritable_startup_file(&home.join("notes.txt")));
    let in_home = inputs(home);
    assert!(!in_home.unwritable_startup_file(&home.join(".zshrc")));
    assert!(!in_home.unwritable_startup_file(&home.join(".bash_profile")));
}

/// #620: the words a command writes to, and not the ones it only reads or
/// runs.
#[test]
fn issue_620_write_targets() {
    let targets = |script: &str| write_targets(&shell(script));
    assert_eq!(targets("echo x >> ~/.zshrc"), vec!["~/.zshrc"]);
    assert_eq!(targets("printf x>>~/.zshrc 2>&1"), vec!["~/.zshrc"]);
    assert_eq!(
        targets("cmd &> log; tee -a a b < in"),
        vec!["log", "a", "b"]
    );
    assert_eq!(targets("cp -r src ~/.config/fish"), vec!["~/.config/fish"]);
    assert_eq!(targets("sed -i 's/a/b/' f g"), vec!["f", "g"]);
    assert_eq!(targets("dd if=x of=y"), vec!["y"]);
    assert_eq!(targets("FOO=1 sudo touch t"), vec!["t"]);
    for script in [
        "cat ~/.zshrc",
        "tee out < ~/.bashrc",
        "cd .git/hooks && ln -s ~/.local/bin/pre-commit",
        "~/.local/bin/tool --flag",
        "git commit -m 'document ~/.zshrc'",
        "echo hi 2>&1",
    ] {
        assert!(
            targets(script)
                .iter()
                .all(|target| !target.contains(".zshrc")
                    && !target.contains(".bashrc")
                    && !target.contains("pre-commit")),
            "{script}: {:?}",
            targets(script)
        );
    }
    assert_eq!(
        targets("cmd >&out <>rw; mv ~/.zshrc x"),
        vec!["out", "rw", "~/.zshrc", "x"]
    );
    assert_eq!(
        write_targets(&[
            "env".to_string(),
            "bash".to_string(),
            "-c".to_string(),
            "echo x >> ~/.zshrc".to_string(),
        ]),
        vec!["~/.zshrc"]
    );
    assert_eq!(targets("bash -c 'tee -a ~/.zshrc < in'"), vec!["~/.zshrc"]);
    // Not through a shell, `>` in an argument is text.
    assert_eq!(
        write_targets(&[
            "git".to_string(),
            "commit".to_string(),
            "-m".to_string(),
            "x >~/.zshrc".to_string(),
        ]),
        Vec::<String>::new()
    );
    // Not through a shell: one simple command.
    assert_eq!(
        write_targets(&["tee".to_string(), "-a".to_string(), "/u/.zshrc".to_string()]),
        vec!["/u/.zshrc"]
    );
}

/// #584: the word a command writes that no grant lifts (the project's
/// `.codex` and `.agents`, the Corbanu home), when it writes no path a grant
/// lifts: through `..`, a link as the last part, and a Corbanu home reached
/// through a link, whose `worktrees` folder stays writable.
#[cfg(unix)]
#[test]
fn issue_584_only_never_writable_words() {
    use codex_protocol::permissions::NetworkSandboxPolicy;
    let base = tempfile::tempdir().unwrap();
    let base = base.path().canonicalize().unwrap();
    let work = base.join("work");
    let home = base.join("corbanu-home");
    for folder in [
        work.join(".codex"),
        work.join(".agents"),
        work.join(".git/hooks"),
        work.join("sub"),
        home.join("worktrees/x"),
    ] {
        std::fs::create_dir_all(&folder).unwrap();
    }
    std::os::unix::fs::symlink(work.join(".codex/config.toml"), work.join("settings")).unwrap();
    let linked_home = base.join("linked-home");
    std::os::unix::fs::symlink(&home, &linked_home).unwrap();
    let cwd = AbsolutePathBuf::from_absolute_path(&work).unwrap();
    let profile = PermissionProfile::workspace_write_with(
        &[],
        NetworkSandboxPolicy::Restricted,
        /*exclude_tmpdir_env_var*/ true,
        /*exclude_slash_tmp*/ true,
    );
    let word = |codex_home: &Path, script: &str| {
        let denials = ReadDenials::for_turn(
            codex_home,
            codex_home,
            &cwd,
            std::slice::from_ref(&cwd),
            &profile,
        );
        only_never_writable_word(
            &shell(script),
            &work,
            /*user_home*/ None,
            codex_home,
            &denials,
        )
    };
    for (script, expected) in [
        ("printf x > .codex/config.toml", Some(".codex/config.toml")),
        (
            "printf x >> sub/../.agents/notes.md",
            Some("sub/../.agents/notes.md"),
        ),
        ("printf x > settings", Some("settings")),
        (
            "cp notes.txt .agents/notes.md && tee .codex/a",
            Some(".agents/notes.md"),
        ),
        // A git hook beside it: Grant once still lifts that one.
        (
            "printf x > .git/hooks/pre-commit; printf x > .codex/a",
            None,
        ),
        ("printf x > notes.txt", None),
        ("cat .codex/config.toml", None),
    ] {
        assert_eq!(word(&home, script).as_deref(), expected, "{script}");
    }
    // A session working in a Corbanu worktree (#583): the worktree's own
    // `.codex` stays never writable (the most specific entry wins), its
    // other files are writable; also when the home is reached through a link.
    for spelled in [&home, &linked_home] {
        let worktree = spelled.join("worktrees/x");
        std::fs::create_dir_all(worktree.join(".codex")).unwrap();
        let cwd = AbsolutePathBuf::from_absolute_path(&worktree).unwrap();
        for codex_home in [&home, &linked_home] {
            let denials = ReadDenials::for_turn(
                codex_home,
                codex_home,
                &cwd,
                std::slice::from_ref(&cwd),
                &profile,
            );
            let word = |script: &str| {
                only_never_writable_word(
                    &shell(script),
                    &worktree,
                    /*user_home*/ None,
                    codex_home,
                    &denials,
                )
            };
            assert_eq!(
                word("printf x > .codex/a").as_deref(),
                Some(".codex/a"),
                "{spelled:?} {codex_home:?}"
            );
            assert_eq!(
                word("printf x > notes.md"),
                None,
                "{spelled:?} {codex_home:?}"
            );
        }
    }
    // The Corbanu home itself, also spelled through a link, but not its
    // writable `worktrees` folder (#583), whichever spelling is used.
    for codex_home in [&home, &linked_home] {
        let into_home = format!("printf x > {}/notes.md", linked_home.display());
        assert!(word(codex_home, &into_home).is_some(), "{codex_home:?}");
        for spelled in [&home, &linked_home] {
            let into_worktrees = format!("printf x > {}/worktrees/x/notes.md", spelled.display());
            assert_eq!(
                word(codex_home, &into_worktrees),
                None,
                "{codex_home:?} {into_worktrees}"
            );
        }
    }
}

/// #902: a test harness can only lengthen the lookups' bound; it is 2 s
/// otherwise. #997: this crate's unit tests always get the test bound.
#[test]
fn issue_902_unhurried_lookups_only_lengthen_the_bound() {
    let outside_unit_tests = || lookup_timeout_in(/*unit_test*/ false);
    assert_eq!(LOOKUP_TIMEOUT, Duration::from_secs(/*secs*/ 2));
    assert_eq!(outside_unit_tests(), LOOKUP_TIMEOUT);
    unhurried_lookups_for_tests(/*enabled*/ true);
    let unhurried = outside_unit_tests();
    let unit_test = lookup_timeout();
    unhurried_lookups_for_tests(/*enabled*/ false);
    assert_eq!(unhurried, TEST_LOOKUP_TIMEOUT);
    assert_eq!(unit_test, TEST_LOOKUP_TIMEOUT);
    assert!(TEST_LOOKUP_TIMEOUT > LOOKUP_TIMEOUT);
    assert_eq!(outside_unit_tests(), LOOKUP_TIMEOUT);
    assert_eq!(lookup_timeout(), TEST_LOOKUP_TIMEOUT);
}

/// #997: a lookup slower than the production bound still finishes in this
/// crate's unit tests, so a loaded machine does not drop the explanations
/// and withheld grant offers they check.
#[tokio::test]
async fn issue_997_unit_test_lookups_outlast_the_production_bound() {
    let slow = off_thread(|| {
        std::thread::sleep(LOOKUP_TIMEOUT + Duration::from_millis(/*millis*/ 500));
        Some("found")
    });
    assert_eq!(slow.await, Some("found"));
}
