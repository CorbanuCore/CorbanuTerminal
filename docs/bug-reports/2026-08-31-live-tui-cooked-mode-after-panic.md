---
title: "Live TUI becomes input-dead after terminal mode is restored by a surviving panic"
status: in_progress
priority: P0
reported: 2026-08-31
affected_version: "Corbanu Terminal 0.1.35"
affected_commit: "3e6df48bbd"
component: "Rust TUI terminal ownership, panic handling, and child lifecycle"
fix_branch: "fix/live-tui-panic-terminal-ownership"
fix_worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/fix-live-tui-panic-terminal-ownership"
---

# P0 bug-fix request: live TUI left in cooked mode

## Summary

A live Corbanu Terminal process can survive a background or otherwise contained
panic after its global panic hook has restored the terminal to canonical,
echoing mode. The TUI remains rendered and the process remains alive, but its
event reader still expects raw terminal input. Keystrokes are then buffered or
echoed by the kernel instead of being delivered as TUI key events, making the
screen appear irreparably frozen.

This is a terminal-ownership defect, not a provider timeout or a stuck command.
Terminal restoration must be tied to the actual lifetime of the foreground TUI,
not to every invocation of the process-wide panic hook.

The retained defunct direct child observed in the same process is a separate
lifecycle path with a confirmed source: TUI startup discarded the
`internal-gpu-controller` child handle without retaining a waiter. It is
included in this bounded repair, but remains independent of terminal recovery.

## Product authorization and change class

This repair is a **bounded fix**: it restores the already-authorized live TUI
without adding a user goal or changing a security, financial, persistence, or
compatibility boundary.

- Product-spec heading: **Product definition**.
  Requirement excerpt: “let the user direct it conversationally”.
- Product-spec heading: **Shipping MVP — LIVE**.
  Requirement excerpt: “Rust, Apache-2.0, Linux/macOS/Windows, the `corbanu`
  command”.

## Production observation

The failure was observed on 2026-08-31 in tmux session `trading_prod`, pane 0,
while running:

```text
Corbanu Terminal 0.1.35
binary: corbanu-debug-kimi-3e6df48bbd
model: corbanu/kimi-k3 high
tty: /dev/pts/14
```

The visible sequence was:

1. The user submitted `is this workign`.
2. The model ran `fly status -a tasknodeofficial-dev` successfully.
3. The model ran `fly logs -a tasknodeofficial-dev ...` successfully.
4. The user interrupted the turn.
5. Corbanu displayed `Conversation interrupted` and returned to a rendered
   composer, but subsequent input was not processed.

The rollout record proves that the provider and tool lifecycle had completed:

| UTC timestamp | Recorded event |
| --- | --- |
| `00:34:30.693` | Final `fly logs` function output recorded successfully |
| `00:34:32.596` | `<turn_aborted>` recorded |
| `00:34:32.602` | `turn_aborted` completed with reason `interrupted` |

There was no running tool child holding the turn open after the abort.

## Live process evidence

After the apparent freeze:

- The tmux pane was alive, active, and not in copy mode.
- The Corbanu process remained alive and retained its rollout and state
  database descriptors.
- The TUI remained rendered at an idle composer.
- `/dev/pts/14` reported `isig icanon iexten echo`, with canonical input and
  local echo enabled.
- Healthy Corbanu panes on the same host reported raw-mode settings including
  `-isig -icanon -iexten -echo`.
- Repeated Ctrl-C input was handled or echoed by the terminal rather than
  reaching Corbanu's key-event path.

This directly explains the input jam: the live application and the kernel no
longer agree about who owns terminal input.

## Failure boundary

`tui::set_panic_hook` installs a process-wide hook that calls
`restore_after_exit()` for every panic outside the special scoped-credential
case:

```rust
pub(super) fn set_panic_hook() {
    let hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        if codex_vault::scoped_credential_callback_active() {
            return;
        }
        let _ = restore_after_exit();
        hook(panic_info);
    }));
}
```

`restore_after_exit()` calls `restore_common(RawModeRestore::Disable, ...)`,
which disables raw mode. A panic hook runs before Rust or Tokio determines
whether a panic will unwind out of the process, be captured in a task
`JoinHandle`, or be contained by another recovery boundary. Therefore the hook
cannot safely equate "a panic occurred" with "the TUI is exiting."

The same terminal and panic-hook code is unchanged between the affected
`3e6df48bbd` binary and the current 0.1.36 candidate branch at the time of this
report.

## Root-cause assessment

The terminal-state loss is confirmed. The most likely initiating event is a
background or contained panic at or immediately after turn interruption:

1. a task panics;
2. the process-wide hook runs and restores cooked terminal state;
3. Tokio or another recovery boundary contains the panic, so the TUI survives;
4. the TUI continues waiting for crossterm raw-mode key events;
5. canonical buffering and signal handling make the screen input-dead.

The precise panic payload is not recoverable from this incident. No persistent
local TUI log was configured, the rollout format does not record panic-hook
payloads, and tmux scrollback contains no panic line. The missing diagnostic is
an observability defect but does not change the confirmed terminal-state
failure.

Other raw-mode restoration paths were excluded for this observation: there was
no job-control suspend/resume, external interactive program, onboarding
selector, normal process exit, or still-running tool command corresponding to
the state transition.

### Defunct-child root cause

`App::start_gpu_controller` starts `internal-gpu-controller` using
`std::process::Command::spawn()` and immediately discards the returned
`std::process::Child`. The controller intentionally exits immediately when
there is no potentially billable GPU work. On Unix, an exited direct child must
still be waited by its parent; dropping the standard-library child handle does
not reap it.

Local process evidence confirms this lifecycle signature across affected
builds: long-lived Corbanu TUI parents retain one defunct child created within
one second of TUI startup. This timing is independent of model tool calls,
interrupts, panics, and the tmux harness.

## Required repair

Repair the terminal ownership boundary rather than special-casing this prompt,
model, provider, command, or interrupt sequence.

1. Represent active TUI terminal ownership with an RAII guard established as
   soon as raw mode is successfully enabled.
2. Restore terminal modes when that ownership guard is dropped during actual
   top-level unwind or normal exit.
3. Remove the unconditional `restore_after_exit()` side effect from the
   process-wide panic hook, or otherwise prove that only a panic escaping the
   foreground TUI owner can trigger restoration.
4. Ensure a spawned Tokio task panic or any explicitly contained panic cannot
   change terminal modes while the TUI event loop remains alive.
5. Preserve the existing guarantee that a genuinely fatal foreground panic
   restores the parent shell to canonical, echoing mode.
6. Persist a redacted local diagnostic for panic source, thread/task identity,
   and terminal-ownership state so a future incident retains its initiating
   cause without exposing credentials or model content.
7. Retain an asynchronous waiter for the independently spawned GPU controller
   so its one-shot exit is reaped while the TUI stays alive. Do not kill the
   controller when the TUI exits, and do not couple its lifecycle to terminal
   restoration.

## Acceptance criteria

### Automated

- A PTY integration test starts the real TUI, verifies raw/no-echo mode, causes
  a spawned task panic that is observed and contained, and verifies the PTY is
  still raw/no-echo.
- After that contained panic, separately sent text and Enter key events produce
  a visible user submission and the TUI continues processing events.
- A PTY integration test starts an active turn, interrupts it, waits for the
  completed interruption event, and successfully submits a second prompt.
- A fatal foreground panic exits the TUI and leaves the parent PTY in canonical
  echoing mode.
- Normal `/exit`, Ctrl-C exit, suspend/resume, external-editor return, and
  onboarding/error exits continue restoring the correct terminal modes.
- Tests exercise adjacent contained-panic paths rather than matching only the
  incident's literal prompt or `fly` commands.
- The independently spawned GPU controller is explicitly waited and cannot
  remain as a defunct direct child after its one-shot exit.

### True-TUI

- Run the release candidate in tmux with the same inline TUI configuration.
- During a streaming/tool-using turn, interrupt once and wait for
  `Conversation interrupted`.
- Submit a new prompt as distinct text and Enter actions.
- Confirm the prompt is rendered, dispatched, and answered.
- Confirm `stty -a` remains raw/no-echo while Corbanu owns the terminal and
  returns to canonical/echo only after Corbanu exits.

### Observability

- The retained diagnostic distinguishes a fatal foreground panic from a
  contained/background panic.
- It records terminal ownership and mode disposition without logging secrets,
  full prompts, provider credentials, or model output.

## Immediate operator recovery

For the observed pane, another shell can restore the kernel mode without
destroying the session:

```sh
stty raw -echo < /dev/pts/14
tmux refresh-client -S
```

This is an incident workaround only. It does not repair the panic/terminal
ownership boundary and must not be treated as completion evidence.

## Relevant code

| Path | Relevance |
| --- | --- |
| `codex-rs/tui/src/tui.rs::set_panic_hook` | Unconditionally restores terminal state on most panics |
| `codex-rs/tui/src/tui.rs::restore_common` | Disables raw mode and terminal input features |
| `codex-rs/tui/src/tui.rs::restore_after_exit` | Correct exit behavior invoked at an unsafe lifecycle boundary |
| `codex-rs/tui/src/lib.rs::TerminalRestoreGuard` | Existing RAII mechanism that should own real exit restoration |
| `codex-rs/tui/src/app.rs::App::run` | Foreground event-loop and shutdown lifetime boundary |
| `codex-rs/tui/src/app/event_dispatch.rs::App::start_gpu_controller` | Previously discarded the independent controller child handle |
| `codex-rs/tui/src/app/background_child.rs` | Retains the independent asynchronous waiter that reaps the controller |
| `codex-rs/tui/tests/support/tmux_control.rs` | Existing true-PTY/tmux test support suitable for regression coverage |

## Completion evidence required

- Focused unit and PTY integration tests pass on the final tree.
- Existing TUI tests and formatting checks pass.
- True-tmux reproduction passes on Linux.
- Fatal-panic shell restoration is explicitly verified.
- The fix is included in a published Corbanu Terminal release and the affected
  production workflow is retested with that exact binary.

## Repair progress

Local macOS repair is implemented on branch
`fix/live-tui-panic-terminal-ownership` from base `2bcaf8d0b`:

- terminal ownership is activated immediately after raw mode succeeds and is
  released by an RAII guard on normal exit or escaping unwind;
- the process-wide TUI panic hook no longer changes terminal modes;
- a redacted `tui-panics.log` records panic source, thread ID, Tokio task ID,
  ownership state, and later classification as contained/background or fatal
  foreground without persisting the panic payload;
- a real PTY regression contains a spawned Tokio-task panic, proves raw/no-echo
  remains active, then proves the owner restores canonical/echo mode;
- the same regression proves an escaping foreground panic restores
  canonical/echo mode during unwind;
- `just fix -p codex-tui` completed on the repair tree;
- focused result: `1 passed, 3853 skipped` for
  `tui::terminal_ownership_tests::contained_and_fatal_panics_respect_terminal_ownership`;
- the complete `codex-tui` run executed all 3,847 selected tests: 3,806 passed
  and 41 failed in pre-existing snapshot/MkDocs-viewer baseline debt; the
  representative command-popup source and expected snapshot are unchanged
  from `origin/main`, while the terminal-ownership regression passed in the
  same run;
- the final binary entered the real macOS TUI under a PTY and Ctrl-C exited
  cleanly with terminal restoration. A network-backed turn could not be
  started because local Corbanu provider authentication is not configured;
- the GPU controller now runs under a detached asynchronous waiter that reaps
  its exit without killing it when the TUI runtime ends;
- focused child-lifecycle result: `1 passed, 3853 skipped` for
  `app::background_child::tests::detached_wait_task_reaps_the_background_child`;
- native x86_64 Linux qualification ran the final code candidate on Ubuntu
  24.04 and passed both focused regressions: `2 passed, 3850 skipped`. The run
  also exposed and closed a Linux-only PTY assertion-order defect in the new
  test harness before review;
- native x64 Windows qualification on Windows build 26200 compiled the complete
  `codex-tui` test binary with the MSVC toolchain and passed the cross-platform
  detached-waiter regression: `1 passed, 3797 filtered out`. Unix `waitpid`
  zombie semantics and termios ownership remain covered by the native Linux
  and macOS runs;
- a real TUI startup using a process-local placeholder key recorded controller
  PID `71415` as reaped with exit status 0 while parent PID `71395` remained
  alive with zero zombie children. No login or persisted credential changed.

Remaining release evidence:

- resolution or explicit acceptance of the 41 unrelated baseline
  snapshot/MkDocs-viewer failures from the complete `codex-tui` run;
- network-backed interrupt and second-prompt true-TUI run (local Corbanu auth is
  currently unconfigured, so the binary enters provider onboarding);
- named human sign-off, release inclusion, and exact-binary production retest.
