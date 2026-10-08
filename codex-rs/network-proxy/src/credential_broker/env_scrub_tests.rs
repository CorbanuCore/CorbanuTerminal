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

/// Regression for the credential-canary crash (issue #222): `setenv` from one
/// thread reallocated `environ` while another walked it in `take_env_var`,
/// killing the whole test binary with SIGSEGV. Without `env_write_lock` in
/// `take_env_var` this crashes within milliseconds.
#[test]
fn pf_27_s05_take_env_var_is_serialized_with_concurrent_env_writes() {
    let prefix = format!("PF27_S05_RACE_{}", std::process::id());
    let workers: Vec<_> = (0..4)
        .map(|worker| {
            let prefix = prefix.clone();
            std::thread::spawn(move || {
                for round in 0..2_000 {
                    // A fresh name each round grows (and reallocates) `environ`.
                    let name = format!("{prefix}_{worker}_{round}");
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
