//! `pfterminal-acp` — the stable ACP entry point for PFTerminal.
//!
//! ACP (Agent Client Protocol) clients such as Buzz speak JSON-RPC over stdio.
//! PFTerminal does not speak ACP directly; it exposes the Codex app-server
//! protocol. The maintained `codex-acp` adapter translates between the two and
//! honours `CODEX_PATH` to select which Codex-compatible binary it drives.
//!
//! So this binary is deliberately *not* an ACP implementation. It is a thin,
//! stable launcher that:
//!
//!   1. resolves the PFTerminal executable,
//!   2. sets `CODEX_PATH` authoritatively,
//!   3. hands off to `codex-acp`,
//!   4. preserves stdin/stdout/stderr, signals, and the adapter's exit code,
//!   5. writes every diagnostic to stderr.
//!
//! Point 5 is not a style preference. Once the handoff happens, stdout carries
//! ACP protocol frames exclusively — a single stray line on stdout corrupts the
//! stream and the client will fail to parse it.
//!
//! Point 4 is why the Unix path uses `exec()` rather than spawn-and-wait: the
//! adapter *becomes* this process, so Ctrl-C, SIGTERM and exit codes need no
//! forwarding logic that could get them subtly wrong.
//!
//! Why a distinct command rather than telling users to run
//! `CODEX_PATH=pfterminal codex-acp`: ACP hosts key runtime identity off the
//! command. Buzz's built-in Codex runtime is already defined around `codex-acp`,
//! so sharing that executable risks mistaken runtime identity, stale persona
//! pins, and merged usage attribution between two different agents.

use std::env;
use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

/// Environment variable that pins the PFTerminal executable explicitly.
const PFTERMINAL_PATH_ENV: &str = "PFTERMINAL_PATH";
/// Environment variable that pins the adapter executable explicitly.
const CODEX_ACP_PATH_ENV: &str = "CODEX_ACP_PATH";
/// The variable `codex-acp` reads to choose which binary to drive.
const CODEX_PATH_ENV: &str = "CODEX_PATH";

const ADAPTER_BIN: &str = "codex-acp";
const PFTERMINAL_BIN: &str = "pfterminal";

const ADAPTER_INSTALL_HINT: &str = "install it with `npm install -g @agentclientprotocol/codex-acp`, or set \
     CODEX_ACP_PATH to its executable";

fn main() -> std::process::ExitCode {
    let args: Vec<OsString> = env::args_os().skip(1).collect();

    // Handled locally rather than forwarded: `pfterminal-acp --version` should
    // describe *this* launcher and what it resolved, which is the only thing
    // that makes it useful for diagnosing a broken install. ACP itself never
    // sends these flags, so intercepting them cannot affect a live session.
    if let Some(first) = args.first() {
        if first == "--version" || first == "-V" {
            print_version();
            return std::process::ExitCode::SUCCESS;
        }
        if first == "--help" || first == "-h" {
            print_help();
            return std::process::ExitCode::SUCCESS;
        }
    }

    let pfterminal = match resolve_pfterminal() {
        Ok(path) => path,
        Err(err) => {
            eprintln!("pfterminal-acp: {err}");
            return std::process::ExitCode::from(127);
        }
    };

    let adapter = match resolve_adapter() {
        Ok(path) => path,
        Err(err) => {
            eprintln!("pfterminal-acp: {err}");
            return std::process::ExitCode::from(127);
        }
    };

    let mut command = Command::new(&adapter);
    command.args(&args);
    // Authoritative: a CODEX_PATH inherited from the environment would silently
    // point the adapter at a different agent while the client still believes it
    // is talking to PFTerminal.
    command.env(CODEX_PATH_ENV, &pfterminal);

    exec(command, &adapter)
}

/// Replace this process with the adapter, preserving stdio, signals and status.
#[cfg(unix)]
fn exec(mut command: Command, adapter: &Path) -> std::process::ExitCode {
    use std::os::unix::process::CommandExt;
    // `exec` only returns on failure.
    let err = command.exec();
    eprintln!(
        "pfterminal-acp: failed to execute {}: {err}",
        adapter.display()
    );
    std::process::ExitCode::from(126)
}

/// Windows has no `exec`; spawn and propagate the adapter's exit code.
#[cfg(not(unix))]
fn exec(mut command: Command, adapter: &Path) -> std::process::ExitCode {
    match command.status() {
        Ok(status) => match status.code() {
            // Truncate to u8 as ExitCode requires; anything non-zero must stay
            // non-zero so a failure is never reported as success.
            Some(0) => std::process::ExitCode::SUCCESS,
            Some(code) => {
                let narrowed = (code & 0xFF) as u8;
                std::process::ExitCode::from(if narrowed == 0 { 1 } else { narrowed })
            }
            None => std::process::ExitCode::FAILURE,
        },
        Err(err) => {
            eprintln!(
                "pfterminal-acp: failed to execute {}: {err}",
                adapter.display()
            );
            std::process::ExitCode::from(126)
        }
    }
}

/// Locate the PFTerminal executable.
///
/// Sibling-of-self is tried before `PATH` because the two binaries ship
/// together: when a user has several PFTerminal installs, the adapter should
/// drive the one it was installed alongside, not whichever happens to win on
/// `PATH`.
fn resolve_pfterminal() -> Result<PathBuf, String> {
    if let Some(explicit) = env::var_os(PFTERMINAL_PATH_ENV) {
        let path = PathBuf::from(explicit);
        if path.is_file() {
            return Ok(path);
        }
        return Err(format!(
            "{PFTERMINAL_PATH_ENV} is set to {} but that is not a file",
            path.display()
        ));
    }

    if let Some(sibling) = sibling_executable(PFTERMINAL_BIN) {
        return Ok(sibling);
    }

    if let Some(found) = find_on_path(PFTERMINAL_BIN) {
        return Ok(found);
    }

    Err(format!(
        "could not find the `{PFTERMINAL_BIN}` executable next to this binary or on PATH; \
         set {PFTERMINAL_PATH_ENV} to its location"
    ))
}

fn resolve_adapter() -> Result<PathBuf, String> {
    if let Some(explicit) = env::var_os(CODEX_ACP_PATH_ENV) {
        let path = PathBuf::from(explicit);
        if path.is_file() {
            return Ok(path);
        }
        return Err(format!(
            "{CODEX_ACP_PATH_ENV} is set to {} but that is not a file",
            path.display()
        ));
    }

    find_on_path(ADAPTER_BIN)
        .ok_or_else(|| format!("could not find `{ADAPTER_BIN}` on PATH — {ADAPTER_INSTALL_HINT}"))
}

fn sibling_executable(name: &str) -> Option<PathBuf> {
    let exe = env::current_exe().ok()?;
    let dir = exe.parent()?;
    let candidate = dir.join(exe_name(name));
    candidate.is_file().then_some(candidate)
}

fn find_on_path(name: &str) -> Option<PathBuf> {
    let target = exe_name(name);
    env::split_paths(&env::var_os("PATH")?)
        .map(|dir| dir.join(&target))
        .find(|candidate| candidate.is_file())
}

fn exe_name(name: &str) -> String {
    if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    }
}

fn print_version() {
    println!("pfterminal-acp {}", env!("CARGO_PKG_VERSION"));
    match resolve_pfterminal() {
        Ok(path) => println!("  pfterminal: {}", path.display()),
        Err(err) => println!("  pfterminal: NOT FOUND ({err})"),
    }
    match resolve_adapter() {
        Ok(path) => println!("  codex-acp:  {}", path.display()),
        Err(err) => println!("  codex-acp:  NOT FOUND ({err})"),
    }
}

fn print_help() {
    println!(
        "\
pfterminal-acp — run PFTerminal as an ACP agent over stdio.

USAGE:
    pfterminal-acp [ADAPTER_ARGS...]

This is a launcher, not an ACP implementation. It resolves the PFTerminal
executable, sets CODEX_PATH, and hands off to the codex-acp adapter. All
arguments are forwarded to the adapter unchanged.

It is normally started by an ACP client (such as Buzz) rather than by hand;
stdin and stdout carry the protocol.

ENVIRONMENT:
    {PFTERMINAL_PATH_ENV}    pin the PFTerminal executable
    {CODEX_ACP_PATH_ENV}     pin the codex-acp executable
    {CODEX_PATH_ENV}          set by this launcher; any inherited value is overridden

REQUIREMENTS:
    codex-acp must be installed — {ADAPTER_INSTALL_HINT}."
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exe_name_is_platform_appropriate() {
        let name = exe_name("pfterminal");
        if cfg!(windows) {
            assert_eq!(name, "pfterminal.exe");
        } else {
            assert_eq!(name, "pfterminal");
        }
    }

    #[test]
    fn explicit_path_must_be_a_file_not_a_directory() {
        // A directory on PFTERMINAL_PATH is a misconfiguration that must fail
        // loudly; silently falling through to PATH would launch a different
        // agent than the operator pinned.
        let dir = std::env::temp_dir();
        // SAFETY: single-threaded test process.
        unsafe { env::set_var(PFTERMINAL_PATH_ENV, &dir) };
        let result = resolve_pfterminal();
        unsafe { env::remove_var(PFTERMINAL_PATH_ENV) };
        assert!(
            result.is_err(),
            "a directory must not resolve as the binary"
        );
    }

    #[test]
    fn missing_adapter_reports_install_instructions() {
        // SAFETY: single-threaded test process.
        unsafe { env::set_var(CODEX_ACP_PATH_ENV, "/nonexistent/codex-acp-xyz") };
        let err = resolve_adapter().unwrap_err();
        unsafe { env::remove_var(CODEX_ACP_PATH_ENV) };
        assert!(err.contains("not a file"), "unexpected error: {err}");
    }
}
