# RW-TMUX-03 Verification Results

- Date: 2026-08-24
- Worktree: `/Users/travisgood/Documents/ChatGPT/corbanu-tmux-harness`
- Branch: `codex/tmux-control-mode`
- Package base: `20ce0cae2a7efb54afb2d52b82719e064bdd5b9f`
- Status: completed locally; Ubuntu tmux execution is a post-registration CI follow-up

## Delivered

The package adds a private bounded tmux control-mode parser and transport, lazy
control-transcript diagnostics, a two-pane lifecycle contract test, and a typed
migration of the width-resize/restore smoke. It does not change product source,
protocols, persistent state, authorization, shipped documentation, or release
behavior.

Source work was split into reviewable commits:

- `0075b624e` - parse the bounded tmux control protocol.
- `305c8e806` - capture control transcripts and parser errors on failure.
- `017934cff` - add the control transport and two-pane lifecycle proof.
- `21e6a3d3c` - migrate the width-resize smoke to typed tmux support.

## Environment

- macOS 15.6.1, Darwin 24.6.0, arm64.
- tmux 3.7c.
- rustc 1.85.0.
- just 1.58.0.

## Final-Tree Verification

| Check | Result |
|---|---|
| `just fix -p codex-tui` | Passed; only pre-existing unrelated warnings were emitted. |
| `just fmt` | Passed. |
| Parser-focused tests | 6 passed. |
| Full required tmux lane | 22 passed in 9.659 seconds with `CORBANU_TMUX_REQUIRED=1` and zero retries. |
| Owning `codex-tui` integration target | 32 passed in 10.843 seconds; one separate repeated-resize test remained skipped as planned. |
| Snapshot audit | `cargo insta pending-snapshots --manifest-path tui/Cargo.toml` reported no pending snapshots. |
| Failure diagnostics | The induced control-wait timeout produced a bounded transcript and parser-error slot in the lazy bundle. |
| Cleanup | No private socket directory, tmux server, control reader, child process, success artifact, or `LEAK` marker remained. |
| Source review | Manual review and `git diff --check` passed. |

The owning integration run marked the unrelated passing
`vt100_history::em_dash_and_space_word_wrap` test as leaky. The required tmux
lane itself was clean.

Automated autoreview was attempted. Its model process continued returning
healthy heartbeats but produced no review after 33 minutes, so it was stopped
and is recorded as unavailable rather than as a clean review.

## Reliability Sample

The two TMUX-03 scenarios passed 20 consecutive zero-retry runs. Durations in
seconds were:

`10.828, 12.045, 12.358, 12.484, 12.341, 9.154, 8.491, 10.875, 9.377, 10.207, 8.634, 8.212, 8.999, 7.908, 8.710, 8.001, 8.357, 8.614, 8.634, 8.251`

- p50: 8.710 seconds.
- p95: 12.358 seconds.
- Failures, retries, leaked sockets, leaked processes, and success artifacts: 0.

## Live Qualification

A real Corbanu binary was launched with `just codex` inside a private tmux
server and an isolated temporary `CODEX_HOME`. The run used a dummy API key and
local model/provider overrides, handled the repository trust prompt, and made no
provider request.

The driver waited for the ready viewport, sent literal `/status` and Enter as
separate actions, observed the Corbanu v0.1.35 status card with model,
directory, permissions, account, session, and token fields, then sent `/exit`
and observed a clean process exit. One trace log was written at
`/tmp/corbanu-tmux03-live-home.egdA5n/logs/codex-tui.log`.

## Ubuntu Follow-Up

The repository contains `.github/workflows/tmux-smoke.yml`, whose automatic
triggers are pull requests and pushes to `main`. This feature-branch push was
neither, and GitHub returns "workflow not found on default branch" when asked
to dispatch it manually. GitHub therefore did not run the dedicated Ubuntu tmux
lane. A local Docker fallback was also unavailable: Docker Desktop's daemon did
not become healthy after reclaiming 20.4 GiB of stale build output and restarting
the application.

The general Ubuntu build for source commit `21e6a3d3c` was still running at
[GitHub Actions run 32783172064](https://github.com/CorbanuCore/CorbanuTerminal/actions/runs/32783172064)
when this record was written, but that workflow does not execute the tmux lane.
RW-TMUX-04 must verify the dedicated workflow on its first eligible pull-request
or main-branch run. This result makes no Ubuntu tmux pass claim.

## Deferred Scope

- Migrate the remaining ignored repeated-resize test.
- Run the dedicated Ubuntu tmux workflow through an eligible trigger.
- Adopt the proven typed driver incrementally in `qa/orchestrate_tui_matrix.sh`.
- Keep product-specific terminal qualification in each owning product sprint.
