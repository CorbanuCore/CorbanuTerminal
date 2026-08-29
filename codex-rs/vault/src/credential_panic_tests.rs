use std::panic::catch_unwind;
use std::process::Command;
use std::sync::mpsc;

use super::*;

#[test]
fn scoped_credential_panic_guard_restores_nested_and_unwound_scopes() {
    assert!(!scoped_credential_callback_active());
    {
        let _outer = ScopedCredentialPanicGuard::enter();
        assert!(scoped_credential_callback_active());
        assert!(
            catch_unwind(|| {
                let _inner = ScopedCredentialPanicGuard::enter();
                panic!("synthetic nested credential panic");
            })
            .is_err()
        );
        assert!(scoped_credential_callback_active());
    }
    assert!(!scoped_credential_callback_active());
}

#[test]
fn scoped_credential_panic_hook_is_thread_local_and_preserves_other_panics() {
    const CHILD: &str = "CORBANU_PF13_PANIC_GUARD_CHILD";
    const SECRET: &str = "sk-synthetic-panic-hook-canary";
    const ORDINARY: &str = "ordinary-panic-still-visible";
    if std::env::var_os(CHILD).is_some() {
        std::panic::set_hook(Box::new(|info| eprintln!("{info}")));
        let (ready_tx, ready_rx) = mpsc::channel();
        let (finish_tx, finish_rx) = mpsc::channel();
        let worker = std::thread::spawn(move || {
            let _guard = ScopedCredentialPanicGuard::enter();
            ready_tx.send(()).expect("signal guarded scope");
            finish_rx.recv().expect("wait for ordinary panic");
            assert!(catch_unwind(|| panic!("{SECRET}")).is_err());
        });
        ready_rx.recv().expect("wait for guarded scope");
        assert!(catch_unwind(|| panic!("{ORDINARY}")).is_err());
        finish_tx.send(()).expect("release guarded thread");
        worker.join().expect("guarded thread");
        assert!(catch_unwind(|| panic!("after-guard-panic-visible")).is_err());
        return;
    }
    let output = Command::new(std::env::current_exe().expect("test executable"))
        .args(["--exact", "credential_panic::tests::scoped_credential_panic_hook_is_thread_local_and_preserves_other_panics", "--nocapture"])
        .env(CHILD, "1")
        .output().expect("isolated panic-hook test");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stdout.contains(SECRET) && !stderr.contains(SECRET));
    assert!(stderr.contains(ORDINARY));
    assert!(stderr.contains("after-guard-panic-visible"));
}
