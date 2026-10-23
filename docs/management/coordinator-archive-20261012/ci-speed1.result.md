I merged PR #319 (merge commit `adbe1b0f8`). On a warm cache, the job went from 50 to 43 minutes on comparable runners, so it is less close to the 60-minute timeout. Most of the time is now spent running the tests, and caching can't reduce that.

**Where the time goes**
- **The slowdown isn't from new code.** Runs fall into two groups: ~33 minutes and ~50 minutes. Docs-only PRs at the same commit land in both, and the logs show the same runner image in different regions. It's most likely slower runner hardware (inferred, not confirmed). The same 70 tests run each time and take 1173s on the fast runners and 1884s on the slow ones.
- **The workflow compiled most of the code twice.** The workflow's own build step built a different set of packages than the build inside `just test`. That made Cargo rebuild about 170 crates, including codex-core, codex-tui and codex-cli. On the slow runners, compiling took 1135s in total.

| Run | Compiling (s) | Running the tests (s) | Whole job |
|---|---|---|---|
| Before, slow runner | 1135 | 1884 | 50 min |
| Before, fast runner | ~780 | 1173 | 33 min |
| After, empty cache (#319) | 956 | 1860 | 47 min |
| After, warm cache (#319, second run) | 665 | 1858 | 43 min |

Both runs on the PR landed on the slow runners, so compare them with the 50-minute row. The warm run reused 89% of compiled crates from the cache.

**What changed in `tmux-smoke.yml`**
- The build step now builds exactly the same packages as the build inside the test script, then builds the test binary separately. The builds inside `just test` now finish in about 3s and 1s.
- Compiled crates are cached with sccache, the same way `rust-ci-full-nextest-platform.yml` does it. Pushes to main save the cache; a PR saves only when it found nothing to restore. The cache is about 1.7 GB.
- The tests themselves are unchanged: the same command, `--retries 0` and `CORBANU_TMUX_REQUIRED=1`.
- **A failing test still fails the job.** Nothing in the test step can let it pass: `just test` returns the test runner's exit code, the shell stops on a non-zero exit, and the cache steps run separately afterwards.

**For Travis**
1. **The real fix for the test step is one small code change, which I didn't make.** About 28 provider tests each compute a SHA-256 of the ~630 MB debug `corbanu` binary, using the `sha2` crate built without optimisation. I measured one hash at 27.7s on this Mac, and 1.9s with the crate optimised (same result). Setting `opt-level = 3` for `sha2` under `[profile.dev.package.sha2]` and `[profile.test.package.sha2]` (as `scrypt`/`salsa20` already do) should cut roughly 10–15 minutes from each run. It changes how every build compiles, so it needs your approval. My recommendation is to do it.
2. **Some cache misses are unexplained.** 194 crates missed the cache even though only the workflow file had changed between the two runs. Probably something early in the build isn't reproducible, but I didn't find the cause.
3. **Cache storage is full.** The repo was already at 10.3 GB of GitHub's 10 GB cache limit, and this adds about 1.7 GB per push to main. Older cache entries will be evicted sooner.
4. **One unpinned action.** `orchestrate-tui-matrix.yml` uses `Swatinem/rust-cache@v2` without pinning it to a commit, unlike the rest of the repo.

I removed the worktree; no local build directory was created.