//! D392 (b): the runner account's `SeAssignPrimaryTokenPrivilege` lets
//! anyone who can sign in as it become SYSTEM, and any program running as
//! the user can read its password, so setup makes it only for a user who is
//! an administrator anyway: when setup runs as that user (and as the user
//! signed in to its session). A standard user's setup runs as the
//! administrator whose password was typed into the UAC prompt; here, the
//! elevated test process sets Codex up for a standard user it creates. (The
//! session check, which an honest `real_user` doesn't reach, has its own
//! unit test in the setup helper.) That setup makes no runner account, and the elevated
//! sandbox works as before #1800: each runner as its command's account,
//! with the D324 isolation between commands.
//!
//! A normal session can't run the setup (UAC): it reuses the standard
//! user's setup an elevated run recorded right before, in the `d392`
//! folder of `CODEX_PF27S06_SETUP_SEED`.

use super::*;

/// The standard user the elevated test sets Codex up for.
const STANDARD_USER: &str = "CodexD392Standard";

/// A codex home set up for a standard user, or `None` to skip.
fn standard_user_home(name: &str) -> Option<TempDir> {
    // Bazel gives unit tests no staged sandbox helpers (#691).
    if option_env!("BAZEL_PACKAGE").is_some() {
        return None;
    }
    let home = sandbox_home(name);
    let seed = std::env::var_os(SETUP_SEED_ENV).map(|seed| PathBuf::from(seed).join("d392"));
    if crate::setup::is_elevated().expect("elevation") {
        crate::window_station::tests::stage_sandbox_helpers();
        set_up_for_standard_user(home.path());
        if let Some(seed) = seed {
            fs::create_dir_all(&seed).expect("seed dir");
            for (file, dir) in SETUP_FILES {
                fs::copy(home.path().join(dir).join(file), seed.join(file))
                    .expect("record the standard user's setup");
            }
        }
        return Some(home);
    }
    let Some(seed) = seed.filter(|seed| {
        SETUP_FILES
            .iter()
            .all(|(file, _)| seed.join(file).is_file())
    }) else {
        assert!(
            std::env::var_os(REQUIRE_ENV).is_none(),
            "sec-win-d392 needs an elevated run or a recorded setup in {SETUP_SEED_ENV}\\d392"
        );
        eprintln!("sec-win-d392: no elevated run and no recorded setup; skipped");
        return None;
    };
    for (file, dir) in SETUP_FILES {
        let dir = home.path().join(dir);
        fs::create_dir_all(&dir).expect("seed dir");
        fs::copy(seed.join(file), dir.join(file)).expect("seed the standard user's setup");
    }
    crate::window_station::tests::stage_sandbox_helpers();
    Some(home)
}

/// As an administrator, sets Codex in `codex_home` up for [`STANDARD_USER`],
/// whom it creates for that and removes again.
fn set_up_for_standard_user(codex_home: &Path) {
    let net = system32_exe("net.exe");
    if crate::winutil::resolve_sid(STANDARD_USER).is_err() {
        use rand::Rng;
        use rand::SeedableRng;
        let mut rng = rand::rngs::SmallRng::from_entropy();
        // 14 characters: `net user` asks (and, with no input, fails) before
        // taking a longer one.
        let password = format!("Aa1!{:010x}", rng.r#gen::<u64>() >> 24);
        let output = std::process::Command::new(&net)
            .args(["user", STANDARD_USER, &password, "/add"])
            .output()
            .expect("run net user /add");
        assert!(
            output.status.success(),
            "create {STANDARD_USER}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let root = user_owned_test_root();
    let env_map: HashMap<String, String> = std::env::vars().collect();
    let permissions =
        crate::ResolvedWindowsSandboxPermissions::try_from_permission_profile_for_workspace_roots(
            &PermissionProfile::workspace_write(),
            workspace_roots_for(root.path()).as_slice(),
        )
        .expect("permissions");
    let setup = crate::setup::run_elevated_setup_for_user(
        crate::setup::SandboxSetupRequest {
            permissions: &permissions,
            command_cwd: root.path(),
            env_map: &env_map,
            codex_home,
            proxy_enforced: false,
        },
        crate::setup::SetupRootOverrides::default(),
        STANDARD_USER,
    );
    let _ = std::process::Command::new(&net)
        .args(["user", STANDARD_USER, "/delete"])
        .output();
    if let Err(err) = setup {
        panic!(
            "set up for {STANDARD_USER}: {err:#}\n{}",
            sandbox_log(codex_home)
        );
    }
}

fn users_file(codex_home: &Path) -> crate::setup::SandboxUsersFile {
    let path = crate::setup::sandbox_users_path(codex_home);
    serde_json::from_slice(&fs::read(&path).expect("read sandbox_users.json"))
        .expect("parse sandbox_users.json")
}

/// D392 (b): a standard user's setup records no runner account and why;
/// the setup counts as current, Codex says why, and an elevated command
/// still runs, its runner running as the command's account.
#[test]
fn sec_win_d392_standard_user_setup_makes_no_runner_account() {
    let Some(codex_home) = standard_user_home("d392-standard") else {
        return;
    };
    let _guard = legacy_process_test_guard();
    let users = users_file(codex_home.path());
    let root = user_owned_test_root();
    let cmd = system32_exe("cmd.exe").display().to_string();
    let runtime = current_thread_runtime();
    let (stdout, exit_code) = runtime.block_on(run_elevated(
        root.path(),
        codex_home.path(),
        vec![cmd, "/d".into(), "/c".into(), "echo ok".into()],
        &[],
        /*tty*/ false,
    ));
    let log = sandbox_log(codex_home.path());
    println!(
        "\nsec-win-d392 standard user: runner={:?} standard_user={} exit={exit_code} stdout={:?}",
        users.runner.as_ref().map(|runner| &runner.username),
        users.standard_user,
        stdout.trim()
    );
    pretty_assertions::assert_eq!(
        (
            users.runner.is_none(),
            users.standard_user,
            users.version_matches(),
            crate::setup::standard_user_notice(codex_home.path()).is_some(),
        ),
        (true, true, true, true),
        "a standard user's setup made the runner account\n{log}"
    );
    pretty_assertions::assert_eq!((exit_code, stdout.trim()), (0, "ok"), "{log}");
    assert!(
        log.contains("no runner account (standard user, D392)"),
        "the runner didn't run as the command's account\n{log}"
    );
}

/// D392 (b): an administrator's own setup still makes the runner account,
/// and Codex has nothing to tell them.
#[test]
fn sec_win_d392_administrator_setup_makes_the_runner_account() {
    let Some(codex_home) = elevated_home("d392-admin") else {
        return;
    };
    let _guard = legacy_process_test_guard();
    let root = user_owned_test_root();
    let cmd = system32_exe("cmd.exe").display().to_string();
    let runtime = current_thread_runtime();
    let (stdout, exit_code) = runtime.block_on(run_elevated(
        root.path(),
        codex_home.path(),
        vec![cmd, "/d".into(), "/c".into(), "echo ok".into()],
        &[],
        /*tty*/ false,
    ));
    record_setup(codex_home.path());
    let users = users_file(codex_home.path());
    let log = sandbox_log(codex_home.path());
    pretty_assertions::assert_eq!(
        (
            exit_code,
            stdout.trim(),
            users.runner.as_ref().map(|runner| runner.username.as_str()),
            users.standard_user,
            crate::setup::standard_user_notice(codex_home.path()),
        ),
        (0, "ok", Some(crate::setup::RUNNER_USERNAME), false, None),
        "{log}"
    );
    assert!(
        !log.contains("no runner account (standard user, D392)"),
        "{log}"
    );
}

/// D392 (b): without a runner account, commands keep D324's isolation: an
/// elevated command can't open another workspace's concurrent command's
/// process for any write right or adjust its token. (The `pl` default DACL
/// would give the logon SID every runner shares here full access.)
#[test]
fn sec_win_d392_standard_user_command_cannot_write_to_another_workspaces_process() {
    let Some(codex_home) = standard_user_home("d392-peer") else {
        return;
    };
    let _guard = legacy_process_test_guard();
    let holder_root = user_owned_test_root();
    let root = user_owned_test_root();
    let runtime = current_thread_runtime();
    let (stdout, exit_code) = runtime.block_on(with_elevated_holder(
        holder_root.path(),
        codex_home.path(),
        async |holder_pid| {
            run_elevated(
                root.path(),
                codex_home.path(),
                probe_argv(PROBE_ENTRY),
                &[
                    (PROBE_ENV, "peer".to_string()),
                    (TARGET_ENV, holder_pid.to_string()),
                ],
                /*tty*/ false,
            )
            .await
        },
    ));
    let line = expect_line(&stdout, codex_home.path());
    println!("\nsec-win-d392 standard user peer: exit={exit_code} {line}");
    let missing = missing_parts(&line);
    assert!(
        missing.is_empty(),
        "a standard user's elevated command could open another's process: missing {missing:?} in {line}\n{}",
        sandbox_log(codex_home.path())
    );
    assert!(
        sandbox_log(codex_home.path()).contains("no runner account (standard user, D392)"),
        "the runner didn't run as the command's account"
    );
}
