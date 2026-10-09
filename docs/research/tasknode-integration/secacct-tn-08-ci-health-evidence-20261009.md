# Task Node evidence record: SECACCT-TN-08

Compile the SECACCT-TN-08 CI Health Three-Fix Evidence Record. Task `task_646f61b2412189a65917a65719be72d6`, request `req_46339cf77077ac8a28965e5a41219572f33cd9f005c726efeb6399b1c2785696`. Every PR below is merged to `main` in CorbanuCore/CorbanuTerminal; each merge commit was checked with `git merge-base --is-ancestor <sha> origin/main` at `3254a302fd` (2026-10-09). Repository quotes are copied verbatim from the files named above them (relative links re-pointed to this file's location); PR-description quotes are verbatim from `gh pr view <n> --json body` on 2026-10-09. Task map: [tasknode-task-map.md](../../../qa/initiative-control/p1-security-and-accounting/tasknode-task-map.md).

**Status: all three merged; postmerge-ci on main is green again.** #350 records that postmerge-ci failed on every main run from `9bac4915f3` (2026-10-07) through its own work (84 runs checked); the first postmerge-ci run after #350 merged, run 37945252801 on `a202df29c0` (the #350 merge commit), succeeded, and so did the next one. **Open, not claimed:** #347, #348, #349 and #353 (filed, not fixed) and the proposed PR-time schema-fixtures job (not implemented).

| PR | Merge commit on main | Merged (UTC) | Title |
| --- | --- | --- | --- |
| [#309](https://github.com/CorbanuCore/CorbanuTerminal/pull/309) | `265172beed` | 2026-10-08T18:15:44Z | Fix flaky credential canary proxy-injection-boundary probe (SIGSEGV env race) + failure diagnostics |
| [#319](https://github.com/CorbanuCore/CorbanuTerminal/pull/319) | `adbe1b0f81` | 2026-10-08T21:31:19Z | ci(tmux-smoke): sccache + stop double workspace build |
| [#350](https://github.com/CorbanuCore/CorbanuTerminal/pull/350) | `a202df29c0` | 2026-10-09T14:34:58Z | ci: fix the red postmerge-ci on main (stale schema, transcript race, false nested refusals, registry test) |

Verification output (`gh pr view <n> --json state,baseRefName,mergeCommit,statusCheckRollup`, then `git merge-base --is-ancestor`; "checks" counts the PR's final check conclusions):

```text
#309 MERGED base=main merge=265172beed checks: SKIPPED=10 SUCCESS=31; git merge-base --is-ancestor 265172beed 3254a302fd -> exit 0
#319 MERGED base=main merge=adbe1b0f81 checks: SKIPPED=10 SUCCESS=24; git merge-base --is-ancestor adbe1b0f81 3254a302fd -> exit 0
#350 MERGED base=main merge=a202df29c0 checks: SKIPPED=10 SUCCESS=27; git merge-base --is-ancestor a202df29c0 3254a302fd -> exit 0
```

Issue fixed by #309:

| Issue | State | Closed (UTC) | Title |
| --- | --- | --- | --- |
| [#222](https://github.com/CorbanuCore/CorbanuTerminal/issues/222) | CLOSED | 2026-10-08T18:15:45Z | Flaky: macOS credential canary probe proxy-injection-boundary |

## postmerge-ci on main

`gh run list --workflow postmerge-ci.yml --branch main -L 12 --json databaseId,headSha,status,conclusion,createdAt` on 2026-10-09 (run id, head, status, conclusion, created):

```text
37948678214 3254a302fd completed success 2026-10-09T15:02:13Z
37945252801 a202df29c0 completed success 2026-10-09T14:35:01Z
37932166402 d7846e29d5 completed failure 2026-10-09T12:45:43Z
37930636498 a141b3e749 completed failure 2026-10-09T12:31:48Z
37927067931 38362eaf57 completed failure 2026-10-09T11:58:43Z
37922077153 08034c2946 completed failure 2026-10-09T11:10:15Z
37917695916 d0544c1c91 completed failure 2026-10-09T10:27:18Z
37916251022 2ecf6fdbe8 completed failure 2026-10-09T10:13:16Z
37915390136 c261d7b2e5 completed failure 2026-10-09T10:05:06Z
37911426615 6b94db40f4 completed failure 2026-10-09T09:27:50Z
37906672111 6b4b24829b completed failure 2026-10-09T08:42:59Z
37902459043 9c36c54808 completed failure 2026-10-09T08:01:49Z
```

The whole red stretch, from `gh run list --workflow postmerge-ci.yml --branch main --created '>=2026-10-06' -L 300 --json databaseId,headSha,status,conclusion,createdAt` (139 runs), counted in order of creation: the last success before the red stretch is run 37567149188 on `80869748b2` (2026-10-07T03:31:32Z); from run 37572720073 on `9bac4915f3` (2026-10-07T04:42:08Z) up to the #350 merge there are 83 runs: 79 failure, 4 cancelled, 0 success (#350's description counts 84; this list gives 83). The runs after it: run 37945252801 on `a202df29c0` success, run 37948678214 on `3254a302fd` success.

## #309: credential canary SIGSEGV race (#222)

Verbatim, PR #309 description (https://github.com/CorbanuCore/CorbanuTerminal/pull/309), in full:

> Fixes #222.
>
> **Class:** bounded fix (CI reliability). It restores the authorized env-key scrubbing and canary qualification and changes no security boundary. Product spec, "Required trust boundaries", item 3: "Credentials are referenced by label and resolved only inside a trusted execution boundary." The new failure output is redacted synthetic-fixture test output, which the passing report already stores in full.
>
> ## Root cause
> The `proxy-injection-boundary` probe runs `cargo test -p codex-network-proxy --lib credential_broker`, so every matching test runs in one process on parallel threads. `env_scrub::take_env_var` walks the C `environ` array (`overwrite_in_place`) to zero the key's bytes, and it does this without Rust's env lock, which is private. Three tests in the same binary call `std::env::set_var` with new names, and two of them (the `env_scrub` tests) start at the same moment. When one thread's `setenv` reallocates `environ` while another thread is walking it, the walk reads the freed array and the **whole test binary dies with SIGSEGV**. libtest reports no failing test in that case, and the harness printed only `probe … failed`.
>
> - **Reproduced:** with two threads looping `set_var` and `take_env_var`, the binary segfaults in about 20 ms, every time.
> - **Failure rate** (canary jobs since 2026-09-26, all attempts):
>
>   | OS | Proxy-probe failures | Rate |
>   |---|---|---|
>   | macOS | 7 / 333 | 2.1% |
>   | Linux | 1 / 332 | 0.3% |
>
>   About 2.5% of runs failed on at least one OS. The 2 Windows proxy failures were on Windows broker WIP branches (pf-27-s06/s07). `env_scrub` is Unix-only, so they are a different matter.
> - **Isolated stress:** 180 runs of just this probe in CI produced 1 crash, which matches a race with a window of a few microseconds.
>
> ## Fix
> - `env_scrub`: a crate-level env-write lock that `take_env_var` holds for the read, the walk and the remove. Env-mutating tests go through `set_env_var_for_test`, which takes the same lock. The security assertions are unchanged and there is no retry.
> - Regression test `pf_27_s05_take_env_var_is_serialized_with_concurrent_env_writes`. It runs a stress loop (4 threads × 2,000 rounds of `setenv` and `take_env_var`) in a re-executed child process, so a crash cannot take down the other tests in the binary.
>   - With the lock removed, the test fails cleanly: `race child: ExitStatus(unix_wait_status(11))`.
>   - With the lock it passes in about 0.06 s.
> - Harness diagnostics: when a probe fails, the harness now prints the failed tests, each panic's location and message (expected vs observed), the expected tests that were not reported ok, and the crash/stderr tail when there is no test verdict. The output is redacted, and the details are also written to `credential-canary-failure.json`, which is uploaded as an artifact. Other harness changes:
>   - A timeout names the command and prints a redacted, scanned output tail.
>   - The secret scan now sees through ANSI colour codes and also matches `ghp_` and `github_pat_`. I checked the passing reports from all 3 OSes and none of them trips the wider pattern.
>
> Known limit (documented in `env_scrub.rs`): writers outside this crate, and C-level `getenv`, are still not serialized with the walk.
>
> ## Also fixed: Windows `command timed out`
> The new diagnostics named the command on this PR's first run. The **candidate `cargo build`** hit the 900 s default. In 7 of the 9 earlier timeout logs, the failure lands about 15 minutes after the step starts, which matches the same cause.
> - Builds now get 60 minutes. That covers the candidate build and a new per-probe `cargo test --no-run`.
> - Test execution keeps its 15-minute limit, so a hung test still fails.
> - With this change, the Windows canary passed on both runs (about 47 and 51 min).
>
> ## Review
> Opus 5.5 High reviewed three rounds:
> 1. **Request changes.** All fixes landed in commit 2: the child-process repro, pretty_assertions diffs, ANSI handling and redaction.
> 2. **Approve.** The non-blocking follow-ups landed in commit 3.
> 3. **Approve** the build/test timeout split.
>
> Follow-ups, not in this PR:
> - Core's test binary also mixes `set_var` with `take_env_keys`, so the same crash can happen there.
> - No clippy guard yet against a plain `std::env::set_var` in this crate. A per-crate `clippy.toml` would replace the workspace one.

## #319: tmux smoke caching

Verbatim, PR #319 description (https://github.com/CorbanuCore/CorbanuTerminal/pull/319), in full:

> Ubuntu tmux smoke: test step 25–42 min, bimodal by runner region/hardware (docs-only PRs hit both modes), not by commit.
>
> **Changes**
> - The explicit `codex-cli` build used a different package set than the helper build in `scripts/isolated_rust_tests.py`, so feature unification changed and `just test` rebuilt ~170 crates (codex-core/tui/cli…). Now the workflow builds the exact helper package set once and prebuilds the test binary; inside `just test` both builds are now no-ops (3s / 1s).
> - Local-disk sccache (0.12, via actions/cache), same fallback pattern as rust-ci-full-nextest-platform.yml. Main saves; PRs save only on a cold miss (cache ~1.7 GB).
> - Tests unchanged: `just test -p codex-tui --test all tmux --retries 0`, `CORBANU_TMUX_REQUIRED=1`; a failing test still fails the step and the job.
>
> **Timings (build = all cargo compile, incl. the rebuild inside the test step)**
> | run | region | build | test execution | job |
> |---|---|---|---|---|
> | before 37827059534 (slow HW) | eastus | 1135s | 1884s | ~52 min |
> | before 37799919613 (fast HW) | westus2 | ~780s | 1173s | ~33 min |
> | after, cold 37835897102 | westus | 956s | 1860s | ~48 min |
> | after, warm 37841988033 | centralus | 665s | 1858s | ~43 min |
>
> Warm sccache: 89% hit rate (1598 hits / 194 misses).
>
> **Remaining cost is test execution.** ~28 provider-journey tests each SHA-256 the ~630 MB debug `corbanu` binary (`register_evidence` in provider_convergence.rs / provider_management.rs) with `sha2` built at opt-level 0: 27.7s per hash locally vs 1.9s with `sha2` at opt-level 3 (same digest). Suggested follow-up (product tree, not in this PR): `[profile.dev.package.sha2]` + `[profile.test.package.sha2]` `opt-level = 3`, like scrypt/salsa20.

## #350: postmerge-ci red on main

Verbatim, PR #350 description (https://github.com/CorbanuCore/CorbanuTerminal/pull/350), lines 1-1:

> postmerge-ci (full Linux tests; PR checks don't run it) failed on every main run from 2026-10-07 04:42 (9bac4915f3) to now: 84 runs, last green 80869748b2. Every failure was in rust-ci-full `Tests — ubuntu-24.04` shards 1/4 and 2/4. Windows clippy (#317), musl clippy and shards 3/4 and 4/4 passed. Logs from all 84 runs were checked.

Verbatim, PR #350 description (https://github.com/CorbanuCore/CorbanuTerminal/pull/350), section "Failures, causes, fixes":

> ## Failures, causes, fixes
>
> | # | Failing test(s) | First bad | Class | Fix |
> |---|---|---|---|---|
> | 1 | `codex-app-server-protocol schema_fixtures_tests::stable_precomputed_exports_match_schema_fixtures` (53 runs) | 4f09d7af99 (#249 added `TurnContextItem.security_level`) | stale fixture | regenerated with `just write-app-server-schema` (not edited by hand) |
> | 2 | `codex-cli::nested_launch pass_mode_runs_exec_with_aggressive_enforced` (63 runs, every run once tests ran) | da75ee8676 (#231 registry read-only) | test precondition, Linux only | see below |
> | 3 | `codex-core::all suite::unified_exec::unified_exec_formats_large_output_summary` (18 final failures, 3 retry passes) | flaky from 8a8afb2384 (docs-only merge) | real race | see below |
> | 4 | ~120 app-server/exec/tui tests in 10 runs; 1-3 app-server plugin/rate-limit/remote-control tests in many others | sporadic | real bug (environment-triggered) | see below |
> | 5 | `codex-core config::schema::tests::config_schema_matches_fixture` (64 runs) | 9bac4915f3 (#230) | stale fixture | already fixed on main by #338 (5a1aa27291) |
> | 6 | `pf_23_s02_persistence_files_become_read_only`, `pf_30_s01_source_envelopes_label_tool_output_and_keep_human_prompt` (7 runs) | 6b1c8b8873 | Linux regression | already fixed on main by #250 |
>
> **2. Nested pass-mode test.** #231 made Aggressive verification require the Aggressive-homes registry to exist when it is inside the workspace. The test sets the account home to `workspace/no-account`, so the registry is inside the workspace, but nothing created it. In real use the origin's own Aggressive launch creates it (`LaunchPlan::prepare`). The test now creates it the same way. This is not a weaker test: the nested run still verifies the registry is there and read-only. macOS hid the failure because the temp folder is behind `/var -> /private/var`, so `registry.starts_with(cwd)` compared unresolved and resolved paths. The test now resolves its root, so macOS checks the same thing Linux does (without the registry line it fails on macOS too).
>
> **3. Unified-exec transcript race.** The end event's `aggregated_output` came from a transcript filled by a broadcast subscriber (capacity 64). That subscriber starts after the process does, and it drops chunks on `Lagged`. On a loaded runner it kept the head and tail of a 1.3 MB output but lost the middle, so the omission marker was missing. The process's output task now writes the transcript before broadcasting each chunk (local and exec-server paths). The streaming task only emits deltas. New unit test `transcript_keeps_output_the_streaming_task_never_received` (fails without the fix).
>
> **4. False "started by an agent command" refusals.** Protected-command read denials include missing Corbanu homes such as `~/.pfterminal`. On Linux the sandbox masks a missing denied path by binding an empty file there, and bubblewrap creates that file on the host. I reproduced this in ubuntu:24.04: `bwrap --dev-bind / / --ro-bind-data 3 $HOME/.pfterminal …` leaves a 0-byte read-only regular file. If the sandbox parent is killed, the file stays. Nested-launch detection read that file as unreadable state (enforced Aggressive, refuse), and both "sandboxed away" probes returned ENOTDIR, so it counted as an Aggressive origin. Every launch on the account was refused (`…Aggressive is enforced (/home/runner/.pfterminal/security_level.toml)`). That outlives a crashed sandbox on a real Linux machine too. Fix in `security-level`: a candidate home whose metadata says it is not a folder is skipped. If its metadata can't be read, it is still a candidate, because that is what a sandbox denial looks like. Denied existing homes are masked as folders. New test `a_file_in_place_of_a_home_is_not_an_origin`. The placeholder side effect itself is #348.

Verbatim, PR #350 description (https://github.com/CorbanuCore/CorbanuTerminal/pull/350), section "Not fixed here (issues filed, all passed on retry)":

> ## Not fixed here (issues filed, all passed on retry)
> - #347 `tmux_corbanu_env_aliases_restart_account_read_and_legacy_daemon_recovery`: first try failed in 19/84 runs.
> - #349: nine other one-off retry-masked flakes, with run IDs.
>
> No test is skipped or ignored.

Verbatim, PR #350 description (https://github.com/CorbanuCore/CorbanuTerminal/pull/350), section "Catching this earlier":

> ## Catching this earlier
> - Implemented (cheap), split out to #353 because editing `postmerge-ci.yml` triggers a from-source V8 canary build: a `report` job keeps one `postmerge-red` issue open while main is red and closes it when the newest main commit is green.
> - Proposed (not implemented; costs PR minutes): a PR-time `schema fixtures` job when `codex-rs/protocol/**`, `app-server-protocol/**`, `config/**` or `features/**` change. It would run `cargo test -p codex-app-server-protocol --lib schema_fixtures` and the core config-schema test. Both stale fixtures above fail on macOS too, so even a macOS-only job would have caught them.
> - Proposed: macOS PR tests should create temp folders through resolved paths, so path-prefix checks behave as on Linux (see 2).

The review result, from the "Gate" section of the same description (quoted lines only): "Review: one independent read-only review (Opus 5.5 High, via installed `corbanu exec`). Verdict: approve with nits; no correctness or security problem found." The rest of that section (Linux dispatch runs 37926500685 and 37931465215) is summarised, not quoted, because it names a private host address.

## Open, not done

| Issue | State | Closed (UTC) | Title |
| --- | --- | --- | --- |
| [#347](https://github.com/CorbanuCore/CorbanuTerminal/issues/347) | OPEN | — | Flaky on Linux CI: tmux_corbanu_env_aliases_restart_account_read_and_legacy_daemon_recovery (19/84 postmerge runs) |
| [#348](https://github.com/CorbanuCore/CorbanuTerminal/issues/348) | OPEN | — | Linux sandbox creates placeholder files in the user's home (~/.pfterminal) for missing denied Corbanu homes |
| [#349](https://github.com/CorbanuCore/CorbanuTerminal/issues/349) | OPEN | — | Retry-masked flaky tests on Linux postmerge-ci (Oct 7-9) |
| [#353](https://github.com/CorbanuCore/CorbanuTerminal/issues/353) | OPEN | — | ci: report a red main from postmerge-ci |

- Proposed, not implemented: a PR-time schema-fixtures job; macOS temp folders through resolved paths.
- #309 follow-ups: core's test binary also mixes `set_var` with `take_env_keys`; no clippy guard against plain `std::env::set_var` in the crate.
- #319: remaining cost is test execution (`sha2` at opt-level 0); suggested follow-up not done.
