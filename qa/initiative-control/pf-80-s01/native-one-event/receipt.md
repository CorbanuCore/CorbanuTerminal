# PF-80-S01 bound one-event engine — frozen uncommitted receipt

Verified 2026-09-12 UTC. Product initiative, PF-80-S01 remains in_progress.
Product heading: **Internal delivery control — TO BUILD**; “Use sequential sprints per initiative”.
Plan: docs/plans/active/initiative-delivery-control.md.
Sprint: docs/sprints/current/initiative-delivery-control/pf-80-s01-delivery-control.md.
Contract: docs/research/tasknode-integration/native-one-event-allocation.md (read completely).
Skill: /Users/Neo/.codex/skills/corbanu-terminal-development/SKILL.md;
root/Rust policies read; existing allocation followed, parent retains shared ledgers.
Worktree: /Volumes/CorbanuDrive/Corbanu/worktrees/tasknode-pf80-s01-20260911.
Branch: workstream/tasknode-pf80-s01-20260911.
Clean launch/final HEAD: 9ae04495c20570308bad4bcc0bf5db9c2c46f989.
Allocation ancestor verified: c33d47f6ccd00a64fcb05a057472d8ca9f0139d4.
First-party read-only source HEAD verified: 40d2df72710a644f33a2b30831061e8265716db0.

## Candidate and boundary

Literal Rust changes: delivery_send.rs (268 lines), delivery_send_tests.rs (451),
lib.rs (+2 lines: cfg(test) private registration only), all in codex-rs/tasknode-session/src/.
Evidence is this receipt and SHA256SUMS; manifest names every file except itself.
Total: 784 added, 0 removed (268 engine, 451 tests, 2 registration, 63 evidence); 333 non-test lines.
The normal-build OFF boundary is the cfg(test) registration; no production constructor.
All new authority construction and transport implementation live in sibling fixtures.
Actual Client::for_session/identity and ActiveSession expiry checks use synthetic sessions.
The skill guided allocation/governance checks; no shared plan or finished-product docs changed.
Accepted delivery_goal, Client, auth, CLI/TUI, tracker, dependencies and locks are unchanged.

## Final-tree verification

Rust commands from codex-rs with RUSTUP_TOOLCHAIN=1.95.0 and CARGO_NET_OFFLINE=true.
Installed compiler: rustc 1.95.0 (59807616e 2026-04-14). No direct cargo test invocation.
just test -p codex-tasknode-session delivery_send: 13 passed, 42 filtered; b4de5660-fb50-43a6-9fb0-276bf938ae45.
just test -p codex-tasknode-session delivery_goal: 9 passed, 46 filtered; 3aa87740-d05b-4271-a51e-b9dc3129a5a4.
just test -p codex-tasknode-session: 55 passed, 0 skipped; 9146ad7c-dbb0-4798-ac8a-b931ec836a23.
python3 docs/plans/check.py: active 3/3; python3 docs/sprints/check.py: current 115, archived 122.
git diff --check passed; unchanged preparation/Client verified with git diff --exit-code.
Guarded just fmt ran with sandbox-exec denying workspace writes except the three allocated Rust files
and denying network; CARGO_NET_OFFLINE=true, UV_OFFLINE=1, RUSTUP_TOOLCHAIN=1.95.0.
Rust formatting succeeded; recipe exit 1: Python tried to rewrite eight inherited initiative_control
files; the guard denied every write. uv also warned about exclude-newer parsing. No repair attempted.
Initial engine build failed at a fixture Result::unwrap requiring Debug (zero tests ran).
Corrected only the fixture response setup; final scoped rustfmt --edition 2024 --config skip_children=true
ran on the two new Rust files before all three successful test runs. Stable imports_granularity warning retained.

## Contract coverage and limitations

13 tests include 98 separate preflight negatives (7 facts x 14 missing/unknown/denied/duplicate/binding cases),
explicit default/named profile fences, actual account/origin/token rotation, empty/noncanonical identity,
full approved payload equality and Python digest, selection/mapping/staleness, expiry, cancellation,
compile-time non-Clone check, second-use denial, timeout/failure, HTTP holds and strict receipt fields.
Credential canaries remain absent from typed outcomes; API key appears only in the recorded fixture wire body.
Missing/invalid expiry is conservatively held here; native expiry behavior remains unchanged. No new TTL.
After exchange starts, cancellation or a changed fence yields unknown with original event ID/digest.
No rollback, durable exactly-once or safe-retry claim; 401 suggests existing profile relink without doing it.
The preparation's unverified-live-authority advisory is retained even on synthetic success.
No real credentials, live targets, external network, new sockets, enrollment, posting or recovery wiring.
Only the unmodified full-crate loopback test used a socket. No commits/pushes, reviewers, subagents or model calls.
No live credential-provenance/authority, task acceptance/completion, human, TUI, live-repository, benchmark,
release or whole-sprint completion claimed. Parent owns review, integration and later outer adapter allocation.
