# Issue #997 load reproduction results (2026-10-20)

Issue: [#997](https://github.com/CorbanuCore/CorbanuTerminal/issues/997)

## Outcome

The reported failure did not reproduce. The target test passed 476 focused runs
under CPU load and passed in nine observed concurrent-suite attempts. A final
bounded attempt at the exact revision named in the issue did not finish its fresh
build within 45 minutes, so it ran zero test attempts.

This report records negative evidence. It does not claim that #997 is fixed, and it
does not provide a first failing target run because no target failure occurred.

## Attempt 1: focused test under CPU load

- Workflow run: [39380314740](https://github.com/johanstrubs/CorbanuTerminal/actions/runs/39380314740)
- Workflow revision: `3593cccc944afe6c24088e5d48a8d242e55b9495`
- Retry policy: disabled
- Load stage 1: six busy-loop workers
- Load stage 2: twelve busy-loop workers
- Result: 476 target passes, zero target failures; the 150-minute workflow limit
  ended the run.

Command repeated:

```console
just test -p codex-core --lib -E 'test(issue_621_622_interpreter_denials_are_explained_and_listed)' --retries 0
```

## Attempt 2: reporter's concurrent suite under CPU load

- Workflow run: [39386596913](https://github.com/johanstrubs/CorbanuTerminal/actions/runs/39386596913)
- Workflow revision: `af20f6680c1d7fb2a3698cc436187ba00233d6cb`
- Retry policy: disabled
- Background load: three busy-loop workers
- Result: the target passed in eight complete suites and also passed in suite 9.
  Suite 9 exited nonzero because an unrelated MCP test timed out.

Exact command:

```console
just test -p codex-core --lib --features developer-accounting -E 'test(security::) | test(network_approval) | test(mcp_tool_call) | test(session::tests::)' --retries 0
```

The complete unedited output from suite 9 is preserved in the public workflow
artifact. Its relevant final lines were:

```text
Summary [ 106.487s] 814 tests run: 813 passed, 1 timed out, 2407 skipped
TIMEOUT [  60.006s] (514/814) codex-core mcp_tool_call::tests::approve_mode_skips_guardian_in_every_permission_mode
error: test run failed
error: recipe `test` failed on line 82 with exit code 100
```

The target line in that same output was:

```text
PASS [   0.394s] (371/814) codex-core security::protected_reads::tests::issue_621_622_interpreter_denials_are_explained_and_listed
```

## Attempt 3: exact reported revision with a hard cap

- Workflow run: [39443679995](https://github.com/johanstrubs/CorbanuTerminal/actions/runs/39443679995)
- Environment artifact: [12277632340](https://github.com/johanstrubs/CorbanuTerminal/actions/runs/39443679995/artifacts/12277632340)
- Reported source revision: `7e66e5a4dcd2737d2900642b2b613531916ef3b4`
- Host: Intel macOS 15, four logical CPUs
- Bound: one runner, 45 minutes, no more than three suite attempts
- Result: checkout and toolchain setup passed, but the fresh build did not finish
  before the 45-minute timeout. The suite loop ran zero times.

The planned commands, in order, were:

```console
git rev-parse HEAD
rustc --version
cargo --version
cargo nextest --version
just test -p codex-core --lib --features developer-accounting -E 'test(issue_621_622_interpreter_denials_are_explained_and_listed)' --retries 0
just test -p codex-core --lib --features developer-accounting -E 'test(security::) | test(network_approval) | test(mcp_tool_call) | test(session::tests::)' --retries 0
```

Recorded environment:

```text
7e66e5a4dcd2737d2900642b2b613531916ef3b4
rustc 1.95.0 (59807616e 2026-04-14)
cargo 1.95.0 (f2d3ce0bd 2026-03-21)
cargo-nextest 0.9.148 (cd1d6d546 2026-10-07)
release: 0.9.148
commit-hash: cd1d6d5467322dbc0d39a80cb671d000ac302798
commit-date: 2026-10-07
host: x86_64-apple-darwin
logical CPUs: 4
```

## Race hypothesis from the source

`denial_explanation` sends protected-path matching to
`tokio::task::spawn_blocking` through `off_thread`. `off_thread` wraps that work in
`tokio::time::timeout`. Under enough scheduler or filesystem contention, the
blocking lookup can lose to the timeout. `off_thread` then returns `None`, and
`denial_explanation` has no explanation to return.

The two racing pieces are therefore the blocking protected-path lookup and its
timeout. This mechanism matches the issue's observed missing explanation, but the
runs above did not capture it, so it remains a source-based hypothesis rather than
a confirmed reproduction.

## Interpretation

Zero failures in 476 focused trials does not prove a zero failure rate and cannot
predict the number of runs required to see one. The concurrent-suite evidence is
more representative of the original report, but it also produced no target
failure. Reproducing the original observation may depend on machine-specific load,
filesystem contention, runner scheduling, or another condition not preserved in
the issue report.
