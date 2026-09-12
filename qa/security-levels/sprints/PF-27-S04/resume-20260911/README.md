# PF27 resumed service-stage qualification

## Frozen scope and authority

User request: resume security development after accepted PF13 main landing.
Travis separately approved exactly one sixth Fable 5.1 High review via
Corbanu/TMUX. Five prior broker reviews are spent. No privileged installation,
principal/ACL changes, real Vault migration or protected activation is approved.

Owner `/root`; branch `feat/security-broker-resume-20260911`; exact worktree and
base `d870c92dab2bf3fbb602dc3b8447fe9f3534aecb` are recorded in PF-27-S04 and the
active P0 plan. Change class: existing product initiative, service construction
substage only. Product heading **Non-negotiable controls**: “Permit agents to
reference credentials only by label; resolve them solely inside the trusted
execution boundary.” No new user-facing feature is claimed.

Nine original commits `db141e9cb..cd7457da7` replayed cleanly. The resumed service
and Linux transport source are byte-identical to the staged source. Only the
new service package's lock version is reconciled to workspace 0.1.42. The scope
is 12 Rust/build paths, 648 added lines and one removed line, chiefly optional
synthetic fixtures/tests. The production composition library is 75 lines, with
small module/binary declarations and EINTR changes. No architecture expansion,
provider transport implementation, installer or new native bootstrap is added.

## Final-tree evidence

Source checkpoint `fdf4f6caa`; `codex-rs` tree
`d4bea9e30406e4b2442fb7d5094d461eef119088` on both local and clean remote checkouts.
RTX worktree `/home/travis/worktrees/security-broker-resume-20260911`.
Fix service/broker, format and Cargo/Bazel lock update passed without any
tracked remote source changes. The build lock serialized all shared-target work.

| Check | Outcome |
| --- | --- |
| Default service tests | 1/1; production rejects even synthetic-looking arguments with exit 78 |
| Feature-gated synthetic subprocess tests | 6/6 |
| Full broker/Vault/network-proxy suites | 338/338; earlier library-only selection also 322/322 |
| Focused Core broker/client configuration | 6/6; 2,411 unrelated tests skipped |
| Build default and explicitly named fixture binaries | Pass |
| Private TMUX production-denial and synthetic lifecycle rehearsal | Pass; keys/Enter sent separately, six real subprocess cases repeated |
| Plan/sprint checkers | Pass; three active plans, sole security allocation PF27-S04 |

Scripts [qualify-rtx.sh](qualify-rtx.sh) and [tmux-lifecycle.py](tmux-lifecycle.py)
record exact commands; [rtx/](rtx/) retains logs, keys and captures. The full
crate run was executed separately after the initial library-only selection;
the reproducible script now includes the full run. One TMUX invocation was
started before the candidate copy completed and was interrupted while waiting
for the missing executable. Its capture is retained as `tmux-premature-launch.txt`;
the script now checks candidate existence before creating a terminal. The
subsequent recorded pass used the completed package. No Rust process was stopped.

Immutable binaries under `/home/travis/security-round5/evidence/pf27-resume-20260911/candidate/`:

- Service SHA-256 `89c1ee8ccb0c003a18228ed285f8a055fe0d66b7e490bedacb6e0ad0dd4d87da`.
- Fixture SHA-256 `7072e79757e44271b8e2d3c660892c8fd9606fa4c97fe9c54432a15f74477091`.

This is supporting PTY proof of a non-interactive service, not a user-facing
TUI acceptance claim. New code-blind UX design and both live repositories are
not applicable to this narrow internal stage; PF26's eventual candidate-wide
qualification remains mandatory. No new Mac/Windows runtime tests or benchmark
completion are claimed. Default-denial does not establish native isolation.

## Review 6 packet

Review the diff from exact accepted main `d870c92dab2bf3fbb602dc3b8447fe9f3534aecb`
to the frozen branch tip, not the earlier whole security branch. Verify trusted
peer checks before registration/Vault access, binding/grant consistency,
channel teardown, death/restart/replay refusal, absolute deadlines under EINTR,
synthetic feature/argument gating and explicit production unavailability.
Check resumed compatibility with current main without treating unimplemented
native bootstrap, provider streaming or all-OS qualification as completed.
Read-only review only; no nested reviewers, privileged actions or live secrets.
Record findings and classify them against this bounded scope before any fix.

Review result is pending. The prior clean Astra review is historical evidence,
not this sixth review's result. Do not launch another invocation if one is active.

## Remaining boundary

PF20's protected controller was subsequently completed and merged; its contract
can be consumed by a later broker bootstrap step. It does not wire itself into
this service or prove native ownership/handle/peer guarantees. Production data
plane, cached TLS generation replacement, uploads/streams/output gates and
all-platform isolation remain open in PF-27-S04. This substage does not complete
the sprint or authorize service installation.
