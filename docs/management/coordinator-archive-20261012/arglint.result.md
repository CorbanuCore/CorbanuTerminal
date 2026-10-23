I opened PR #148 with all 1,394 violations fixed. The full local macOS lint is now clean, but CI isn't green yet. On Windows the job fails before linting `tui` because of an existing Bazel error that PR #147 fixes. The Linux and macOS jobs were still running after about two hours when I stopped checking.

PR: https://github.com/CorbanuCore/CorbanuTerminal/pull/148 (branch `codex/arg-comment-backlog-20261002`, one commit `366714d444`, based on `origin/main` at `c9f358c0a7`).

**Before and after**
- **Before:** 1,394 unique violations, found by a full local macOS lint of all 948 targets, including test targets. Of these, 1,365 were literal arguments with no comment and 29 had a comment that didn't match the parameter name. All 29 were `/*spend*/` → `/*spend_microunits*/` in `core/src/security/credential_capability_tests.rs`.
- The Windows CI jobs on #147 reported 68 unique violations, all of them also in the local set.
- **After:** the local lint run passes with exit 0 across all 948 targets. That run covers macOS code only. I couldn't run Linux locally, so Linux-only code is checked only by the pending CI job.

**Counts per crate**

| Crate | Fixes |
| --- | ---: |
| core | 458 |
| security-audit | 291 |
| tui | 235 |
| secret-broker | 102 |
| security-policy | 94 |
| vault | 58 |
| provider-auth | 57 |
| tasknode-session | 23 |
| network-proxy | 22 |
| cli | 21 |
| browser-isolation | 19 |
| memories/write | 4 |
| keyring-store | 3 |
| model-provider-info, models-manager | 2 each |
| codex-api, model-provider, app-server-client | 1 each |

**Lint result and checks**
- I applied the lint's own suggested fix at each reported location, then ran `just fmt`.
- A script confirmed the changes are comments only: with comments, whitespace and trailing commas removed, every changed file matches main. The only other differences are 6 places where `just fmt` added or removed braces around a closure or match arm, which doesn't change behavior.
- `cargo check --tests` passes for all 18 changed crates.
- The Windows job on #148 found no lint violations. It failed only because Bazel can't build `tui`, where `provider-auth` is listed twice as a dependency in `codex-rs/tui/BUILD.bazel`. To lint `tui` locally I removed the duplicate temporarily and didn't commit it.
- **Main has two unrelated problems that will keep this lint red until fixed:**
  - the duplicate dependency above, which #147 removes;
  - the Linux and macOS CI jobs on main failed while downloading the LLVM toolchain, so CI never reported their violations. Locally I had to re-download one corrupted Bazel dependency (LLVM) to get my run working.

**Couldn't fix mechanically:** nothing. If the Linux job reports violations in Linux-only code, they can be fixed the same way with a follow-up commit on the branch.

I didn't merge, force-push or touch any other PR, and I didn't read or print any secrets.