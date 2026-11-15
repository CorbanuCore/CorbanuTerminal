//! Main's routing for Corbanu's plain-text copies of protected values: vault credentials and Task
//! Node briefs, each copied on its own blocking task outside the session clipboard worker.
//!
//! Upstream's worker also forwards every copy to a local tmux or Herdr session, which would leave
//! a copied credential in a tmux paste buffer. These copies keep main's order instead: over SSH
//! only the terminal (tmux, else OSC 52) can reach the person's clipboard; otherwise the native
//! clipboard, then on WSL the Windows clipboard, and the terminal only when those fail.

use super::ClipboardLease;
use super::Osc52Target;
use super::is_ssh_session;
use super::is_tmux_session;
use super::is_wsl_session;
use super::osc52_sequence;
use super::tmux_clipboard_copy;
use super::write_osc52;

#[derive(Clone, Copy)]
struct PlainTextEnvironment {
    ssh_session: bool,
    wsl_session: bool,
    tmux_session: bool,
}

/// Copies `text` with main's routing. A terminal send is reported like a native copy without a
/// lease, as on main.
pub(crate) fn copy_plain_text_blocking(text: &str) -> Result<Option<ClipboardLease>, String> {
    let wsl_deadline = std::time::Instant::now() + std::time::Duration::from_secs(/*secs*/ 5);
    #[cfg(not(target_os = "android"))]
    let native_copy = |text: &str| {
        let clipboard =
            arboard::Clipboard::new().map_err(|e| format!("clipboard unavailable: {e}"))?;
        super::arboard_copy(clipboard, text, /*html*/ None)
    };
    #[cfg(target_os = "android")]
    let native_copy = |_text: &str| Err("native clipboard unavailable on Android".to_string());
    copy_plain_text_with(
        text,
        PlainTextEnvironment {
            ssh_session: is_ssh_session(),
            wsl_session: is_wsl_session(),
            tmux_session: is_tmux_session(),
        },
        tmux_clipboard_copy,
        |text| {
            let target = if std::env::var_os("TMUX").is_some() {
                Osc52Target::Tmux
            } else if std::env::var_os("STY").is_some() {
                Osc52Target::Screen
            } else {
                Osc52Target::Direct
            };
            write_osc52(&osc52_sequence(text, target)?)
        },
        native_copy,
        |text| super::wsl::copy(text, wsl_deadline),
    )
}

fn copy_plain_text_with(
    text: &str,
    environment: PlainTextEnvironment,
    tmux_copy_fn: impl Fn(&str) -> Result<(), String>,
    osc52_copy_fn: impl Fn(&str) -> Result<(), String>,
    native_copy_fn: impl FnOnce(&str) -> Result<Option<ClipboardLease>, String>,
    wsl_copy_fn: impl FnOnce(&str) -> Result<(), String>,
) -> Result<Option<ClipboardLease>, String> {
    if text.is_empty() {
        return Err("nothing to copy: the selected content is empty".to_string());
    }
    let terminal_copy = |text: &str| {
        if environment.tmux_session {
            tmux_copy_fn(text).or_else(|tmux_err| {
                osc52_copy_fn(text)
                    .map_err(|osc_err| format!("tmux clipboard: {tmux_err}; OSC 52: {osc_err}"))
            })
        } else {
            osc52_copy_fn(text)
        }
    };
    if environment.ssh_session {
        // Over SSH the native clipboard is the remote machine's.
        return terminal_copy(text)
            .map(|()| None)
            .map_err(|err| format!("terminal clipboard copy failed over SSH: {err}"));
    }
    let native_err = match native_copy_fn(text) {
        Ok(lease) => return Ok(lease),
        Err(err) => err,
    };
    if environment.wsl_session {
        match wsl_copy_fn(text) {
            Ok(()) => return Ok(None),
            Err(wsl_err) => {
                return terminal_copy(text).map(|()| None).map_err(|terminal_err| {
                    format!(
                        "native clipboard: {native_err}; WSL fallback: {wsl_err}; terminal fallback: {terminal_err}"
                    )
                });
            }
        }
    }
    terminal_copy(text).map(|()| None).map_err(|terminal_err| {
        format!("native clipboard: {native_err}; terminal fallback: {terminal_err}")
    })
}

#[cfg(test)]
#[path = "plain_text_tests.rs"]
mod tests;
