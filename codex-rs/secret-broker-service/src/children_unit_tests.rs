use super::*;
use pretty_assertions::assert_eq;
use std::process::Command;
use std::time::Duration;
use std::time::Instant;

#[test]
fn pf_27_s01_reaper_spawn_failure_returns_both_owned_children() {
    let journal = Command::new("/bin/sleep").arg("30").spawn().unwrap();
    let policy = Command::new("/bin/sleep").arg("30").spawn().unwrap();
    let expected = (journal.id(), policy.id());
    let result = TrustedChildRun::capture_with_reaper(
        NonZeroU64::new(1).unwrap(),
        journal,
        0,
        policy,
        0,
        |_, _| Err(io::Error::other("synthetic thread resource exhaustion")),
    );
    let Err((error, mut journal, mut policy)) = result else {
        panic!("capture must fail before taking ownership");
    };
    assert_eq!((journal.id(), policy.id()), expected);
    assert_eq!(error.to_string(), "synthetic thread resource exhaustion");
    journal.kill().unwrap();
    policy.kill().unwrap();
    journal.wait().unwrap();
    policy.wait().unwrap();
}

#[test]
fn pf_27_s01_early_drop_uses_preallocated_reaper() {
    let journal = Command::new("/bin/sleep").arg("30").spawn().unwrap();
    let policy = Command::new("/bin/sleep").arg("30").spawn().unwrap();
    let run = TrustedChildRun::capture(NonZeroU64::new(1).unwrap(), journal, 0, policy, 0)
        .unwrap_or_else(|(error, mut journal, mut policy)| {
            let _ = journal.kill();
            let _ = policy.kill();
            let _ = journal.wait();
            let _ = policy.wait();
            panic!("capture failed: {error}");
        });
    let reaped = Arc::clone(&run.reaped);
    drop(run);
    let deadline = Instant::now() + Duration::from_secs(3);
    while reaped.load(Ordering::Acquire) != 2 {
        assert!(
            Instant::now() < deadline,
            "early-drop children were not reaped"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}
