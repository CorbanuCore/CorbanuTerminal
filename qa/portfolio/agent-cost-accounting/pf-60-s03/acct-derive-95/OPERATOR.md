# Re-verify PF-60-S03 from the repository

This checks the retained acceptance evidence; it cannot establish complete bills,
independent functional acceptance, or permission to ship. Start by reading
[the acceptance decision](../acct-acceptance-75/acceptance.md), especially the open
scope-zero finding and historical timeout. Treat recorded conclusions as claims.

**Get a clean copy.** Use Python 3 and Git. From the repository you were given,
record `git rev-parse HEAD` as the candidate commit. Clone that repository with
full history, not a shallow clone or exported files; check out that exact commit.
Use a destination you own and replace the placeholders below. Uncommitted edits
are not included. In this disposable clone only, set the integration ref to the
candidate so the replay audits the intended commit rather than an older branch:

```sh
git clone --no-local --no-checkout /path/to/supplied-repository /path/to/audit-copy
cd /path/to/audit-copy
git checkout --detach <candidate-commit>
git update-ref refs/heads/integrate/management-workstreams-20260911 <candidate-commit>
git status --porcelain --untracked-files=all
q=qa/portfolio/agent-cost-accounting/pf-60-s03
python3 -B "$q/acct-inventory-79/verify_acceptance.py"
python3 -B "$q/acct-inventory-79/check_verifier.py"
python3 -B "$q/acct-selfcheck-84/check_current_tip.py"
```

**Read the outputs, including exit status.** Initial Git status must be empty.
The verifier must end with `BASELINE MATCH` and
`RESULT agreement=20 disagreement=0 unavailable=3 exit=2`; process exit 2 is the
expected incomplete evidence result. It derives the bounded numeric, inventory,
capture and historical test claims. The control checker must exit 0 with all
13 cases passed. The separately invoked replay must exit 0, report
`stdout_matches_prior_replay: true`, unchanged clean status and identical resolved
commits, with nested acceptance/control exits 2/0 and empty stderr. Its
`run_directory` retains raw streams and receipt. It compares exact stdout against
committed expected bytes and their SHA-256; it is **not run automatically** by the
verifier or CI. None of these checks proves arbitrary prose or human approval.

**The three unavailable artifacts** are `codex`, `codex-code-mode-host` and
`rmcp_test_server` in `acct-fitness-76/package/`. They are ignored unsigned local
debug binaries, excluded for repository storage cost (about 680 MiB), not a
licensing prohibition. The manifest/build log are retained; their historical
binary bytes, modes and hashes cannot be verified from this clone. Read
[the exclusion record](../acct-baseline-81/package-exclusion.md). Rebuilding does
not prove historical identity; closing availability needs the exact retained
package from immutable artifact storage. Do not turn missing bytes into passes.

**Fresh Rust gates, if required.** Read
[credential isolation](../../../../../docs/development/test-isolation.md) and
[build setup](../../../../../docs/install.md#source-build). Install the pinned Rust
toolchain, `just` and `cargo-nextest`; populate dependencies before an offline
build. From the clean clone root, use one dedicated target for all commands:

```sh
export CARGO_TARGET_DIR="$PWD/$q/acct-derive-95/target"
export NEXTEST_TEST_THREADS=4 CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0
export CARGO_PROFILE_DEV_DEBUG_ASSERTIONS=true CARGO_PROFILE_TEST_DEBUG_ASSERTIONS=true
cd codex-rs
cargo build --locked --offline -p codex-cli --bin codex -p codex-rmcp-client -p codex-code-mode-host --bins
just test -p codex-core accounting
just test -p codex-core accounting --features codex-core/developer-accounting
just test -p codex-tui usage
```

Run each command separately; inspect its exit/log before starting the next.
Require the isolation banner, nonzero runs and zero failures/timeouts: the
recorded baseline is 124/124 default, 127/127 feature and 91/91 TUI (342 overlapping
executions), with 3545/3545/4078 skipped. Record fresh counts and failure names;
later passes never erase the retained round-66 timeout. Stop successors on any
native credential prompt or live-profile read; never authorize it.

**If replay disagrees:** preserve the candidate SHA, command, exit, receipt and
raw stdout/stderr before retrying. Read the acceptance exit table: 3 means drift
or refused baseline; 1/traceback is failed execution/check; missing history/ref is
a prerequisite failure. Compare changed files and hashes with the reference.
Restore accidental edits or send intentional claim/evidence changes to the
integrator for review, refreshing inventories and expected output/digest together
only after justification. Never regenerate expectations merely to pass. Keep
scope-zero, settlement, independent package/platform/profile and live-repository
gaps open until their separate evidence and authority requirements are met.
