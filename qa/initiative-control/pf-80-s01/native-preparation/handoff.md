# PF-80-S01 native preparation — frozen uncommitted candidate

Product initiative; **Internal delivery control — TO BUILD**, “Use sequential
sprints per initiative”. Plan: `docs/plans/active/initiative-delivery-control.md`.
Sprint PF-80-S01 remains in_progress; manager owns review, ledgers and integration.
Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/tasknode-pf80-s01-20260911`.
Branch: `workstream/tasknode-pf80-s01-20260911`.
Actual launch and final HEAD: `265bf0c3e164e497d172f1a8e5a56bf4cd54ed46`.
Recorded allocation base: `0415a00dc3d3ee55a96662920c3eedbc0f0d4838`.

## Delivered boundary

Private `delivery_goal` module, 325 implementation lines, 309 test lines; lib.rs
has exactly one added private module declaration. No production callers or new
dependencies. `prepare` takes one event JSON string, explicit selected ID,
full-payload digest, expected sprint/workspace/tasks and injected UTC time.
The returned payload has private fields and read-only accessors, plus advisory
blockers. No authentication, filesystem, posting, queue or transport argument.
No generated identity, clock read, normalization, capture, sync, retry or sender.

ID binding retains `cc-` plus SHA-256 of Python `json.dumps(event_without_id,
sort_keys=True).encode()`. Full payload binding is SHA-256 of Python
`json.dumps({"event": event}, sort_keys=True).encode()`, including the existing ID.
This digest excludes credentials and is not the server's compact stableJson hash.
ASCII escapes, surrogate pairs, DEL, backslash, quotes and separators are covered
by two literal Python-generated fixtures checked in Rust and against real Python
checked_event/prepare and the pinned first-party server validator offline.
First-party commit: `40d2df72710a644f33a2b30831061e8265716db0`; validator SHA-256:
`a2f11df4adca1abe09b70bc4a0d7ae36118501831b13e884bc9e5de8c7198a7c`.

## Deliberate subset and remaining work

This is not full Python prepare parity. Only canonical 24-byte millisecond UTC
timestamps emitted by event_for are accepted; other Python-accepted ISO forms
need explicit future parity coverage. Secret checks conservatively hold embedded
12-character token prefixes, empty credential assignments, bare PRIVATE KEY
markers and bearer whitespace; exact Python regex acceptance parity is deferred.
Duplicate keys are rejected. PF-76-S01 history and changed mappings reject rather
than return a Python preview blocker; there is no migration or retargeting.
Goal status/active validation follows checked_event without inventing state rules.
Staleness remains advisory after 45 minutes; future skew above five minutes rejects.

Posting OFF, enrollment, pending/retry state, account/origin/profile binding,
entitlement, target ownership/lifecycle, exact live approval and one-attempt native
transport are later allocations. Preparation always retains the unverified-live-
authority blocker. Parent owns independent review; none was invoked here.
No commit/push/merge/release/deploy or live account/credential access occurred.
TUI and live-repository qualification are not applicable to this private pure
boundary. Human acceptance, benchmarks and release qualification are not claimed.

## Verification and environment

See `tests.log` and `SHA256SUMS`. All final native tests followed scoped rustfmt.
`just fmt-check` preflight failed: eight inherited initiative_control Python
files would be rewritten outside allocation. Thus broad `just fmt` was not run;
scoped `rustfmt --edition 2024` formatted only the two allocated Rust files.
Stable rustfmt warned that imports_granularity requires nightly; no toolchain repair.
Preflight uv also created ignored formatter environments (including scripts/.venv)
and emitted an exclude-newer parsing warning. These are tool side effects, not
candidate files; no unrelated tracked files changed. System Python lacked
markdown_it; the existing pinned `.venv-v0D0As/bin/python` ran the Python checks.
No additional dependency installation or repair was undertaken after preflight.
Existing suite server/timer output is mocked fixture output, not deployment.
