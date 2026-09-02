# Tmux TUI harness

Corbanu Terminal's tmux harness drives the real terminal application inside an
isolated pseudo-terminal. It gives contributors a deterministic way to send
actual keys, observe stable rendered frames, exercise pane geometry, and retain
useful diagnostics when an interactive workflow fails.

Use it when the behavior depends on terminal input, rendering, focus, resize,
pane lifecycle, or the integration between a model response and a TUI tool
action. Unit tests and snapshots should still cover the underlying logic and
rendered states. A tmux scenario proves the applicable end-to-end terminal
workflow; mocked providers do not replace live-provider or human acceptance
when a plan or release record requires those separately.

This page is an operating guide, not a second policy source. The repository
root policy remains authoritative for when interactive proof is required and
which flows it must cover. Load the repository's `test-tui` skill before a
qualification run; see [Skills](skills.md) for how repository skills are
discovered.

## Harness map

| Path | Responsibility |
| --- | --- |
| `codex-rs/tui/tests/all.rs` | Single integration-test binary containing the support and scenario modules |
| `codex-rs/tui/tests/support/tmux.rs` | Private tmux server, sessions, panes, input, capture, stable waits, and cleanup |
| `codex-rs/tui/tests/support/tmux_command.rs` | Typed command construction and secret-bearing environment redaction |
| `codex-rs/tui/tests/support/tmux_artifacts.rs` | Lazy failure bundles and bounded registered attachments |
| `codex-rs/tui/tests/support/tmux_process.rs` | Pane/server process ownership and bounded teardown |
| `codex-rs/tui/tests/support/tmux_layout.rs` | Typed horizontal and vertical splits |
| `codex-rs/tui/tests/support/tmux_control.rs` | Optional bounded tmux control-mode client for multi-pane event workflows |
| `codex-rs/tui/tests/suite/` | Product and rendering scenarios |
| `.github/workflows/tmux-smoke.yml` | Required Ubuntu focused smoke lane and failure-artifact upload |

Each `TmuxServer` uses a unique `tmux -L` socket under a temporary directory.
It never targets the developer's default tmux server. Sessions and panes are
addressed by the immutable identifiers returned by tmux, and their processes
are checked during teardown.

## Prerequisites

From the repository root, confirm tmux and the candidate test binary are
available:

```bash
tmux -V
just codex --version
```

The integration tests can also locate a Cargo-provided `codex-tui` binary or an
existing `codex-rs/target/debug/{codex,codex-tui}` candidate. A scenario should
fail with a clear build instruction when its required candidate is absent.

Local runs skip tmux scenarios when tmux is unavailable. Set
`CORBANU_TMUX_REQUIRED=1` whenever absence must be a failure, including CI and
qualification runs.

## Run the harness

Run every focused tmux scenario with retries disabled:

```bash
cd codex-rs
CORBANU_TMUX_REQUIRED=1 \
  just test -p codex-tui --test all tmux --retries 0
```

Run one scenario by its test-name selector:

```bash
cd codex-rs
CORBANU_TMUX_REQUIRED=1 \
  just test -p codex-tui --test all \
  tmux_smoke_single_enter_dispatches_slash_command_and_exits_cleanly \
  --retries 0
```

Use a dedicated failure-artifact location when collecting evidence:

```bash
cd codex-rs
CORBANU_TMUX_REQUIRED=1 \
CORBANU_TMUX_ARTIFACT_DIR=/tmp/corbanu-tmux-artifacts \
  just test -p codex-tui --test all tmux --retries 0
```

The Ubuntu workflow runs the same focused selector on pull requests, pushes to
`main`, and manual dispatches. It uploads the artifact directory only when the
job fails.

## Write a scenario

Add Unix scenarios under `codex-rs/tui/tests/suite/` and register their module
in `suite/mod.rs`. Keep model-provider fixtures and assertions in the scenario;
keep terminal mechanics in `tests/support/`.

A typical scenario follows this order:

1. Call `TmuxServer::should_run` and return cleanly when an optional local host
   lacks tmux.
2. Create a fresh temporary `CODEX_HOME`, configuration, log directory, and any
   non-secret fixtures.
3. Prefer a local mocked model server for deterministic UI/tool behavior. Use a
   live provider only when the acceptance contract specifically requires it.
4. Start `TmuxServer` with a stable scenario name and register safe diagnostic
   files such as `config.toml` and `codex-tui.log`.
5. Create a fixed-size `SessionSpec` and launch the real candidate with
   `CommandSpec`.
6. Set `RUST_LOG=trace` and pass `-c log_dir="<isolated-dir>"` for interactive
   qualification.
7. Wait for a semantic ready marker before sending input.
8. Send literal text with `send_literal`, or an explicit terminal paste event
   with `send_paste`; wait until it is visibly settled, and send Enter
   separately with `send_key(TmuxKey::Enter)`.
9. Use bounded stable waits for visible checkpoints. Do not substitute fixed
   sleeps for readiness checks.
10. Capture the viewport or bounded scrollback when the assertion needs it.
11. Exercise success, cancellation/failure, recovery, and resume paths required
    by the feature contract.
12. Exit through the product workflow, normally `/exit`, and call
    `wait_for_exit` so process cleanup is part of the proof.

The central construction pattern is:

```rust
let tmux = TmuxServer::start("tmux_example_flow")?;
tmux.register_artifact("config.toml", codex_home.path().join("config.toml"));
tmux.register_artifact("codex-tui.log", log_dir.join("codex-tui.log"));

let session = tmux.new_session(
    SessionSpec::new(
        "codex-example-flow",
        TerminalSize::new(120, 40),
        CommandSpec::new(candidate)
            .env("CODEX_HOME", codex_home.path())
            .env("RUST_LOG", "trace")
            .arg("-c")
            .arg(format!("log_dir=\"{}\"", log_dir.display()))
            .arg("--no-alt-screen")
            .arg("-C")
            .arg(&repo_root),
    )
    .current_dir(&repo_root),
)?;

let pane = session.primary_pane();
pane.wait_stable_contains("Corbanu Terminal", Duration::from_secs(15))?;
pane.send_literal("/status")?;
pane.wait_stable_contains("/status", Duration::from_secs(5))?;
pane.send_key(TmuxKey::Enter)?;
pane.wait_stable_contains("Permissions:", Duration::from_secs(15))?;
```

`wait_stable_contains` and `wait_stable_until` require the semantic condition
and two identical matching captures. This prevents the test from acting on a
transient frame. Timeouts include the last viewport and trigger diagnostics.

## Input and capture rules

Literal text and named keys are intentionally different APIs:

```rust
pane.send_literal("Enter")?;       // types five characters
pane.send_key(TmuxKey::Enter)?;    // sends the Enter key
pane.send_paste("/tmp/image.png")?; // writes one bracketed-paste byte sequence
```

Never combine test text and Enter into one tmux command. Wait until the literal
text appears before sending the key; this avoids the race where Enter reaches
the application before the complete command.

Use `send_paste` when the workflow depends on paste semantics rather than raw
characters, including image-path attachment. It writes one bracketed-paste
byte sequence without reading from or mutating the developer's system
clipboard; the target application remains the end-to-end authority for whether
that becomes one paste event. Paste text containing an escape character is
rejected because it could terminate bracketed-paste mode early.

Use `capture_viewport` for the currently rendered terminal and
`capture_scrollback_tail(lines)` when the assertion needs prior output. Keep
captures bounded even when the host has a large tmux history limit.

Use `split_vertical` and `split_horizontal` for geometry workflows. Close the
returned typed pane and assert that the primary pane reaches its restored
semantic layout. Use `attach_control` only when a scenario genuinely needs
tmux control-mode events across panes; ordinary TUI input and rendering checks
should stay with the simpler pane API.

## Failure artifacts

Successful scenarios create no bundle. A tmux command failure, stable-wait
timeout, or panic writes one scenario directory beneath
`CORBANU_TMUX_ARTIFACT_DIR` or `codex-rs/target/tmux-artifacts` by default.

The bundle can contain:

- `manifest.json`
- `reason.txt`
- `viewport.txt`
- `scrollback.txt`
- `pane-metadata.txt`
- `command-log.txt`
- `input-events.txt`
- `dimensions.txt`
- `reproduce.sh`
- a control transcript when control mode was attached
- explicitly registered configuration or log attachments

Registered attachments are capped at 2 MiB. The command recorder redacts
environment assignments whose names contain `API_KEY`, `TOKEN`, `SECRET`,
`PASSWORD`, or `CREDENTIAL`. Literal and paste payloads are recorded only as
their input kind and byte length; their contents are omitted from
`command-log.txt` and `input-events.txt`.

Read `reason.txt` first, then compare `input-events.txt` with the viewport and
scrollback. A literal input followed immediately by a key but absent from the
viewport usually indicates an input-settling race. `pane-metadata.txt` answers
whether the process exited, changed size, or remained alive. Use
`reproduce.sh` from the same candidate tree.

## Secret and canary safety

- Use only generated synthetic canaries. Never use a real provider key, wallet
  secret, recovery phrase, or production credential.
- Store a canary only when the boundary under test requires real encrypted
  storage; otherwise keep it out of the test environment entirely.
- Never register `auth.json`, an encrypted vault, a keyring fallback, or another
  credential-bearing file as a failure attachment.
- Do not place the raw canary in the prompt, mocked model response, tool command,
  assertion message, or report.
- Scan every observable surface relevant to the contract: viewport,
  scrollback, model request/tool-result bodies, logs, rollouts, audit output,
  errors, and artifacts.
- Report a digest or an opaque canary identifier when an evidence record needs
  correlation; do not report the raw value.
- Disable retries for security qualification so an initial leak or race cannot
  be hidden by a later pass.

## Worked example: PF-13-S04 protected vault helper

On 2026-08-26, PF-13-S04 used an ad-hoc tmux scenario to test the protected
credential negative path against the actual candidate TUI and CLI binary. The
scenario did not modify the product and was removed after the run.

For both Moderate and Aggressive, it:

1. Created a separate temporary Corbanu home and trace-log directory.
2. Generated a unique synthetic API-key canary and stored it in the encrypted
   vault under a synthetic label.
3. Wrote the protected security level to the isolated `config.toml`.
4. Started a local mocked Responses server with two deterministic responses:
   first a shell-tool request invoking the candidate's `vault auth-helper`, then
   a final assistant completion marker.
5. Launched the real candidate in a private 140-by-44 tmux PTY with
   `RUST_LOG=trace`, its isolated `CODEX_HOME`, and dedicated `log_dir`.
6. Sent the test prompt and Enter separately.
7. Waited for `vault auth-helper is unavailable under <level>` and the final
   completion marker.
8. Scanned the viewport, 10,000 lines of scrollback, both model requests,
   trace logs, rollouts, and every file in the isolated home for the raw canary.
9. Exited through `/exit` and verified that both private tmux servers and their
   socket directories were removed.

The result was one passing test in 44.242 seconds with zero retries. Moderate
and Aggressive both displayed the expected denial, every canary scan was clean,
and the harness emitted no failure artifact. This is supporting interactive
evidence for the negative path. It does not replace later `/security` profile,
broker-success, live-provider, recovery, or human-acceptance qualification.

## Cleanup and troubleshooting

The normal Rust drop path kills only the scenario's private server and waits
for its recorded pane and server processes. Never use an unscoped `tmux
kill-server` during diagnosis; that command can destroy the developer's default
server.

After a run, these checks should produce no test-owned socket directory or
process:

```bash
find /tmp -maxdepth 1 -type d -name 'cdx-tmux-*' -print
pgrep -af 'codex-tui-test-' || true
```

Common failures:

| Symptom | Check |
| --- | --- |
| Scenario silently skipped | Set `CORBANU_TMUX_REQUIRED=1` and confirm `tmux -V` |
| Candidate unavailable | Build `codex-cli --bin codex` or the required TUI binary |
| Enter appears to do nothing | Wait for the complete literal input to render before `send_key` |
| Stable wait times out | Inspect the last viewport, scrollback, pane metadata, and trace attachment |
| Mocked turn never completes | Verify the mock supplies every expected Responses request, including the post-tool completion |
| Nextest reports a leak | Check recorded pane/server PIDs and rerun the one selector with `--retries 0` before attributing it to tmux |
| Failure bundle contains sensitive material | Stop, remove the artifact from circulation, and fix fixture/attachment/redaction boundaries before rerunning |

## Local tmux scrollback

The harness keeps its own captures bounded, but developers may configure a
large interactive tmux history for newly created windows and panes:

```tmux
set-option -g history-limit 10000000
```

Reload it with:

```bash
tmux source-file ~/.tmux.conf
tmux show-options -g -w history-limit
```

Tmux applies `history-limit` only when a window/pane history buffer is created.
Existing panes retain the capacity allocated when they started; recreate them
when convenient rather than destroying active work merely to enlarge history.
Very large histories can consume substantial memory when panes produce large
amounts of output.

## Evidence checklist

For plan, sprint, or release evidence, record:

- candidate commit and version;
- worktree and test-repository base commit;
- tmux version and platform;
- exact test selector, command, retry count, and duration;
- literal inputs and separately sent keys;
- visible checkpoints and expected failure/recovery behavior;
- mock or live-provider boundary;
- artifact directory or confirmation that a successful run emitted none;
- canary/redaction scan surfaces when secrets are in scope;
- private-server and process cleanup result; and
- any remaining live-repository, provider, platform, or human-acceptance gate.

The repository-wide interactive evidence requirements remain authoritative in
the root policy; see the [AGENTS.md contributor guidance](agents_md.md). The
implementation examples live in `codex-rs/tui/tests/suite/`, and the focused
CI contract lives in `.github/workflows/tmux-smoke.yml`.
