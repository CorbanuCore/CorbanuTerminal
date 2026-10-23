I fixed the canary flake and merged it as **PR #309** (merge commit `265172beed`). Every check was green on head `2fbf680`; each OS's canary passed twice. Issue #222 closed automatically on merge, and I posted the full explanation on it.

**Root cause.** The `proxy-injection-boundary` probe runs all `credential_broker` tests in one process at the same time. `take_env_var` (in `env_scrub.rs`) zeroes a key by walking the C `environ` array, without Rust's own environment lock. Three tests in the same binary call `std::env::set_var` at the same time; when one call grows the environment it moves that array, the walk reads freed memory, and the whole test binary dies with SIGSEGV. The harness then showed only "probe failed". Locally, a two-thread `set_var` + `take_env_var` loop segfaults within about 20 ms, every time.

**Failure rate** (canary jobs since 2026-09-26, including reruns):

| OS | Proxy-probe failures | Rate |
|---|---|---|
| macOS | 7 / 333 | 2.1% |
| Linux | 1 / 332 | 0.3% |

About 2.5% of runs failed on at least one OS. The 2 Windows failures of this probe came from Windows broker work-in-progress branches, not this race (the code involved is Unix-only). Running only this probe 180 times in CI produced 1 crash, as you'd expect from a window that short.

**What changed:**
- **Fix:** a crate-level lock that `take_env_var` holds while it reads, zeroes and removes the variable. Every test that sets an environment variable now goes through a helper that takes the same lock. No security check was weakened and there are no retries.
- **Regression test:** `pf_27_s05_take_env_var_is_serialized_with_concurrent_env_writes` runs the race in a separate child process, so a crash can't take down the other tests. With the lock removed it fails cleanly (child exits with SIGSEGV) every time; with the lock it passes in about 0.06 s.
- **Failure reporting:** a failed probe now prints:
  - the failing tests;
  - each assertion's location with expected vs observed values;
  - expected tests that never reported ok;
  - how many tests started vs reported, plus the last stderr lines if the binary crashed.

  Everything is redacted, scanned for credentials before printing, and also saved as `credential-canary-failure.json`. A timeout now names the command and shows its last output, and the credential scan also catches `ghp_`/`github_pat_` keys and keys split by colour codes.
- **Second flake, also fixed:** the new reporting showed that the Windows `command timed out` failures were the initial `cargo build`, which can take longer than the 900 s limit on Windows runners. Builds now get 60 minutes; running the tests keeps 15 minutes, so a hung test still fails.

**Review:** Opus 5.5 High, three rounds: changes requested, then approved twice. Its findings are fixed in the PR. The reviews are in `.codex-work/workers-20261002/canary-flake-review/`.

**Tests:** the canary's harness tests pass (20), and all 57 network-proxy `credential_broker` tests pass locally. There's no tmux run or demo video; this is a CI fix with nothing user-visible to show.

**Still open (non-blocking):**
- Core's test binary also mixes `set_var` with taking environment keys, so the same crash could happen there.
- Nothing yet stops a future plain `std::env::set_var` in network-proxy. A per-crate lint config would replace the workspace one, so I didn't add it.
- The worst-case canary run can now exceed the job's 120-minute limit; GitHub would then kill the job before the harness prints its diagnosis.
- On Windows, a timeout kills only `cargo`, so a hung test underneath can still run until the job limit.

**Cleanup:** I removed my worktree, build folder and branch, and deleted the temporary stress-test branch. Another worktree, `worktrees/ci-canary-proxy-flake-222` (with a build folder of the same name), is working on the same issue. It isn't mine, so I left it alone; it's now redundant.