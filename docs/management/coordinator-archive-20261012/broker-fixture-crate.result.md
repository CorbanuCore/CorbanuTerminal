I've opened draft PR #149 to `main`: https://github.com/CorbanuCore/CorbanuTerminal/pull/149. It hasn't been merged. On Linux, the moved tests pass and the 44 test names match the old `--features synthetic-fixture` run exactly. CI had only just started when I stopped watching, so the Bazel jobs that were failing haven't reported yet.

The branch is `codex/secret-broker-fixture-crate` at commit `5e5aacf456`, built on `codex/ci-main-baseline-20261002` (#146). Until #146 lands, the PR's diff against `main` also shows #146's commits. The PR body asks Jim Ricketts for security review before it leaves draft.

**Files moved** (from `secret-broker-service/` to `secret-broker-fixture/`, history kept):
- `src/fixture/{main,synthetic,post_exec}.rs`: the `codex-secret-broker-service-fixture` binary, same name.
- `src/probe/*`: the `codex-protected-root-probe` binary, same name.
- `src/launch/**`: all of `launch`. Nothing in production used it, so it's re-exported from the new crate's `src/lib.rs`.
- `tests/support/{subprocess,children_tests}.rs`, `tests/launch_recipe_lifecycle.rs`, `tests/probe_lifecycle.rs`.
- The `linux + synthetic-fixture` cfgs became plain linux-only cfgs; nothing else changed.

**Other changes:**
- **Broker crate:** it no longer has `[features]`, optional deps or any fixture code. `codex-security-audit` and `codex-security-policy` were only used by the fixture binary, so they moved with it; leaving them would make the `cargo shear` CI check fail. `tempfile` became a dev-dependency.
- **Lockfiles:** `Cargo.lock` only moves edges between the two crates; no outside package was added or removed. `just bazel-lock-update` left `MODULE.bazel.lock` unchanged and `just bazel-lock-check` passes.
- **PF-27 scripts:** the 21 scripts that passed `--features synthetic-fixture` now point at the new package. I updated one evidence doc, `service-qualification-boundary.md`, because it describes the current setup. I did not rewrite recorded evidence such as logs, tmux transcripts, `tmux-keys.json` files, review outputs or checksums.

**Newly public items:** none. The broker's public API only shrinks: the five `Synthetic*` types leave it.

**One test change for the reviewer:** the broker's four `root::tests` stay in the broker crate. They need its private `Composition` and `RootHandler` to inject fake roots, and exposing those would widen a security-sensitive surface. They used to spawn the fixture binary as a child process, but a test can't locate another package's binary. They now re-run their own test binary as that child, through an ignored helper test that does exactly what the old helper did: connect, write `R`, wait for EOF. Production code is unchanged. Previously these tests only ran with the feature turned on, so no CI job ran them; now CI does.

**Verification:**
- **Manifest policy check:** passes. `just fmt` is clean.
- **macOS, `cargo check --tests` on both crates:** OK.
- **macOS, `just test -p` on each crate:** builds, then exits 4 with "no tests to run" because every test is Linux-only. That was already true of the broker crate before. With `--no-tests pass` it succeeds.
- **macOS Bazel:** `bazel build --config=clippy` on both packages is OK (20 targets); `bazel test` gave 9 passed, 4 Windows cross-build targets skipped.
- **Linux target check from macOS:** not run. The `x86_64-unknown-linux-gnu` toolchain isn't installed; the RTX run covers Linux natively.

**Linux run (RTX, kernel 7.0.0-31)**, in a fresh clone at `~/corbanu-rtx/broker-fixture-20261002`; `integration` was not touched:
- `cargo check` and `cargo clippy -- -D warnings` on both crates: OK.
- Tests: **44 passed, 2 skipped**. The skips are the existing RTX frozen-artifact test and the new ignored helper.
- The base commit with `--features synthetic-fixture` gave 44 passed, 1 skipped. That run used a second worktree, `~/corbanu-rtx/broker-fixture-20261002-base`, which I left in place.
- The box has no Bazel or `just`, so Linux Bazel is untested until CI finishes.

**Risks:**
- **New CI coverage:** these Linux tests now run in Cargo and Bazel CI for the first time. They need Linux 6.5+ for one socket check, 6.3+ for sealed memory files, `python3`, a non-root runner and `HOME` set. A runner missing any of these will now fail instead of silently skipping.
- **Re-run under Bazel:** the `root::tests` re-run of their own binary works under Cargo on Linux but is unconfirmed under Bazel on Linux.
- **Old test counts:** the PF-27 scripts' hard-coded test counts belong to their recorded runs. Re-running those scripts on this tree will show different counts.
- **Moving `launch`:** if a future production launcher needs the `launch` code, it will have to be moved back into a production crate under its own review.