# Receiving harness: three defects that manufactured failures, 2026-09-17

Three receipts failed on 2026-09-17 and none of them failed for a reason in the
candidate. Each one is recorded here because the cost is always the same: a
harness failure reads exactly like a product defect, and the first instinct is
to blame the change under test.

## 1. A prerequisite binary the suites shell out to was not built

`receive.py` built `codex-cli --bin codex` and `codex-rmcp-client --bins`, but not
`codex-code-mode-host`. The `codex-core` code-mode suites spawn that binary from
the target directory, so four of them failed as plain assertion mismatches -
`assertion failed: (left == right)`, `<1` against `>2` - with nothing in the
output naming a missing program. Paired against base the same four failed, which
is what proved it was the harness rather than the candidate.

Fixed: the prerequisite list now builds all three.

## 2. Unbounded test parallelism manufactures deadline failures

The `codex-core` accounting suites carry internal deadlines of roughly twelve
seconds. At full nextest parallelism on this host, **11 of 124 fail at base**
with `Error: deadline has elapsed`; every one of them passes in about one second
when run alone, and the whole lane passes 124/124 with `NEXTEST_TEST_THREADS=4`.
A failure that appears only under load is not evidence about the change under
test, and 26 of those elapsed deadlines in one log is not a red suite, it is a
saturated machine.

Fixed: rust lanes now cap concurrency at four unless the caller overrides it.

## 3. A long gate must not depend on the shell that started it

The first attempt at the security receipt was killed mid-gate when the session
that launched it ended, leaving the merge commit in the receiving tree with no
receipt and an unreconciled `active.json`. Nothing was lost - the integrator
refuses to proceed until an owner reconciles, which is the behaviour working -
but an hour of gate time was.

Fixed: receipts and paired gates run under a detached `tmux` server.

## Standing rule this reinforces

Never accept "that failure is unrelated". Run the identical lane at the base
commit, in the same tree, on the same target, and compare failure sets. Tonight
that rule separated 13 pre-existing `codex-app-server` failures and 11
pre-existing accounting failures from the two candidates that were, in fact,
clean - and it was also the only reason the one genuinely new failure
(`turn_start_shell_zsh_fork_exec_approval_decline_v2`) could be shown to be a
base flake by rerunning it at base rather than argued away.
