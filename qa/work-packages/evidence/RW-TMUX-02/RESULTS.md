# RW-TMUX-02 verification results

Date: 2026-08-24
Branch: `codex/tmux-artifacts-slash-smoke`
Base: `b7aff3e3bfbeebe7897f8ccb10c569ff18f9eff6`
Artifact stage: `510beb079`
Slash and CI stage: `303ebfe10`
Reproduction selector fix: `1193aded2`
Deterministic process cleanup: `2683005ab`
CI trigger fix: `d4f1343dc`
Status: completed

## Delivered

- Lazy failure bundles for timeout, command error, and panic paths.
- Stable manifest plus reason, viewport, bounded scrollback, pane metadata,
  redacted command log, input ledger, dimensions, reproduction command, and
  registered attachments capped at 2 MiB each.
- Real Corbanu `/model` dispatch using literal text and one separately encoded
  Enter, followed by clean `/exit` shutdown.
- Pane and tmux server PID ownership with bounded teardown waits, preventing
  test-owned processes from outliving the Nextest case that launched them.
- Ubuntu 24.04 workflow with hard tmux availability, zero retries, focused tests,
  runtime logging, failure-only artifact upload, and execution on every pull
  request and push to `main`.

## Final-tree verification

- `just fix -p codex-tui`: passed; only pre-existing warnings reported.
- `just fmt`: passed.
- `just test -p codex-tui --test all tmux --retries 0`: 9/9 passed.
- `just test -p codex-tui --test all --retries 0`: 19/19 passed, two intentional
  ignored resize tests skipped.
- Twenty consecutive `tmux_smoke --retries 0` runs: 20 passed, zero retries,
  zero failure artifacts, zero private socket roots, and zero tmux-server leaks.
  Durations were 6-7 seconds; nearest-rank p95 was 7 seconds.
- Snapshot audit: no pending snapshots.
- Workflow YAML parsed successfully; action references use repository-pinned
  checkout, Rust toolchain, installer, and artifact-upload revisions.
- Final branch autoreview reported no actionable findings after the reproduction
  selector and CI trigger fixes.

## Diagnostic finding

The first repetition series exposed one slash-dispatch timeout that Nextest
retried to a pass. Its new bundle showed `/model` and one Enter in the input
ledger but no rendered command text, proving the key arrived before literal
input had settled. The final test waits for `/model` to appear before sending
its single Enter, also waits for `/exit`, and disables retries in CI. The final
20-run series had no failure or artifact emission.

A later reliability pass exposed an intermittent Nextest `LEAK` marker. The
harness now records pane and server PIDs and waits for both during teardown; the
final focused set and 20-run series were clean. In the broader mixed test target,
Nextest twice assigned one marker to different non-tmux rendering tests. One of
those tests passed 10 isolated runs without a marker, so this is recorded as
pre-existing/concurrent Nextest attribution noise rather than a TMUX-02 leak.

No product source, product documentation, snapshots, credentials, protocol,
persistent state, or release claims changed.
