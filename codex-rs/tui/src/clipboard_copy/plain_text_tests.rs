use super::*;
use pretty_assertions::assert_eq;
use std::cell::RefCell;

#[derive(Debug, Default, PartialEq)]
struct Calls {
    tmux: usize,
    osc52: usize,
    native: usize,
    wsl: usize,
}

fn run(
    environment: PlainTextEnvironment,
    native_ok: bool,
    wsl_ok: bool,
) -> (Result<bool, String>, Calls) {
    let calls = RefCell::new(Calls::default());
    let result = copy_plain_text_with(
        "secret",
        environment,
        |_| {
            calls.borrow_mut().tmux += 1;
            Ok(())
        },
        |_| {
            calls.borrow_mut().osc52 += 1;
            Ok(())
        },
        |_| {
            calls.borrow_mut().native += 1;
            if native_ok {
                Ok(Some(ClipboardLease::test()))
            } else {
                Err("no display".to_string())
            }
        },
        |_| {
            calls.borrow_mut().wsl += 1;
            if wsl_ok {
                Ok(())
            } else {
                Err("no powershell".to_string())
            }
        },
    );
    (result.map(|lease| lease.is_some()), calls.into_inner())
}

fn environment(ssh: bool, wsl: bool, tmux: bool) -> PlainTextEnvironment {
    PlainTextEnvironment {
        ssh_session: ssh,
        wsl_session: wsl,
        tmux_session: tmux,
    }
}

/// A local tmux session gets a successful native copy only: nothing is
/// forwarded to tmux or the terminal (upstream's worker would do both).
#[test]
fn local_tmux_copy_stays_on_the_native_clipboard() {
    let (result, calls) = run(environment(false, false, true), true, true);
    assert_eq!(result, Ok(true));
    assert_eq!(
        calls,
        Calls {
            native: 1,
            ..Calls::default()
        }
    );
}

/// Over SSH only the terminal reaches the person's clipboard: tmux in a
/// tmux session, otherwise OSC 52; the remote native clipboard is skipped.
#[test]
fn ssh_copy_uses_only_the_terminal() {
    let (result, calls) = run(environment(true, false, true), true, true);
    assert_eq!(result, Ok(false));
    assert_eq!(
        calls,
        Calls {
            tmux: 1,
            ..Calls::default()
        }
    );
    let (result, calls) = run(environment(true, false, false), true, true);
    assert_eq!(result, Ok(false));
    assert_eq!(
        calls,
        Calls {
            osc52: 1,
            ..Calls::default()
        }
    );
}

/// The terminal is a fallback: after a failed native copy, and on WSL after
/// the Windows clipboard failed too.
#[test]
fn terminal_is_only_a_fallback_after_native_and_wsl_fail() {
    let (result, calls) = run(environment(false, true, false), false, true);
    assert_eq!(result, Ok(false));
    assert_eq!(
        calls,
        Calls {
            native: 1,
            wsl: 1,
            ..Calls::default()
        }
    );
    let (result, calls) = run(environment(false, true, false), false, false);
    assert_eq!(result, Ok(false));
    assert_eq!(
        calls,
        Calls {
            native: 1,
            wsl: 1,
            osc52: 1,
            ..Calls::default()
        }
    );
    let (result, calls) = run(environment(false, false, false), false, false);
    assert_eq!(result, Ok(false));
    assert_eq!(
        calls,
        Calls {
            native: 1,
            osc52: 1,
            ..Calls::default()
        }
    );
}

#[test]
fn empty_text_is_not_copied() {
    let calls = RefCell::new(0);
    let result = copy_plain_text_with(
        "",
        environment(false, false, false),
        |_| Ok(()),
        |_| Ok(()),
        |_| {
            *calls.borrow_mut() += 1;
            Ok(None)
        },
        |_| Ok(()),
    );
    assert!(result.is_err());
    assert_eq!(*calls.borrow(), 0);
}
