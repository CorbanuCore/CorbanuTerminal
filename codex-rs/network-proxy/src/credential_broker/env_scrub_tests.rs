use super::*;
use pretty_assertions::assert_eq;

#[test]
fn pf_27_s05_take_env_var_returns_and_removes_the_value() {
    let name = format!("PF27_S05_SCRUB_{}", std::process::id());
    set_env_var_for_test(&name, "synthetic-scrub-value");
    let value = take_env_var(&name).expect("value");
    assert_eq!(value.as_slice(), b"synthetic-scrub-value");
    assert_eq!(std::env::var_os(&name), None);
    assert!(take_env_var(&name).is_none());
}

#[test]
fn pf_27_s05_take_env_var_ignores_unset_empty_and_invalid_names() {
    let name = format!("PF27_S05_EMPTY_{}", std::process::id());
    set_env_var_for_test(&name, "");
    assert!(take_env_var(&name).is_none());
    assert_eq!(
        std::env::var_os(&name).as_deref(),
        Some(std::ffi::OsStr::new(""))
    );
    assert!(take_env_var("PF27_S05_NEVER_SET_XYZ").is_none());
    assert!(take_env_var("A=B").is_none());
    assert!(take_env_var("").is_none());
}

const RACE_CHILD_ENV: &str = "CODEX_PF27_S05_ENV_RACE_CHILD";
const RACE_CHILD_TEST: &str = "credential_broker::env_scrub::tests::pf_27_s05_env_race_child_entry";

/// Hammers `take_env_var` with concurrent `setenv`s. Runs only in the child
/// started below, so a crash cannot take down the other tests' process.
#[test]
fn pf_27_s05_env_race_child_entry() {
    if std::env::var_os(RACE_CHILD_ENV).is_none() {
        return;
    }
    let workers: Vec<_> = (0..4)
        .map(|worker| {
            std::thread::spawn(move || {
                for round in 0..2_000 {
                    // A fresh name each round grows (and reallocates) `environ`.
                    let name = format!("PF27_S05_RACE_{worker}_{round}");
                    set_env_var_for_test(&name, "synthetic-race-value");
                    let value = take_env_var(&name).expect("value");
                    assert_eq!(value.as_slice(), b"synthetic-race-value");
                }
            })
        })
        .collect();
    for worker in workers {
        worker.join().expect("worker");
    }
}

/// Regression for the credential-canary crash (issue #222): `setenv` on one
/// thread reallocated `environ` while another walked it in `take_env_var`,
/// and the whole test binary died with SIGSEGV. Without `env_write_lock` in
/// `take_env_var` the child crashes within milliseconds.
#[test]
fn pf_27_s05_take_env_var_is_serialized_with_concurrent_env_writes() {
    let output = std::process::Command::new(std::env::current_exe().expect("test binary"))
        .args([RACE_CHILD_TEST, "--exact", "--test-threads=1"])
        .env(RACE_CHILD_ENV, "1")
        .output()
        .expect("run race child");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success() && stdout.contains("1 passed"),
        "race child: {:?}\n{stdout}\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
}
