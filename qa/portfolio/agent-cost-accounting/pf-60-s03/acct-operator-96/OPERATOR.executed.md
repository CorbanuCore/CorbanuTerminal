# Re-verify PF-60-S03 from the repository

This checks the retained acceptance evidence; it cannot establish complete bills,
independent functional acceptance, or permission to ship. Start by reading
[the acceptance decision](../acct-acceptance-75/acceptance.md), especially the open
scope-zero finding and historical timeout. Treat recorded conclusions as claims.

**Get a clean copy.** Use Python 3, Git and Bash. Run this first block from the
root of the supplied repository. It records HEAD and creates a full-history,
detached clone inside ignored scratch space. The destination must not already
exist; preserve earlier attempts and use another supplied copy for a new run.
Uncommitted edits are not included. Inspect the source status printed here to
identify anything omitted; it is not required to be empty.

```sh
set -eu
q=qa/portfolio/agent-cost-accounting/pf-60-s03
candidate=$(git rev-parse HEAD)
printf 'Candidate: %s\n' "$candidate"
git status --porcelain --untracked-files=all
audit="$PWD/$q/acct-operator-96/target/audit-copy"
git check-ignore -v "$audit"
git clone --no-local --no-checkout . "$audit"
git -C "$audit" checkout --detach "$candidate"
git -C "$audit" update-ref refs/heads/integrate/management-workstreams-20260911 "$candidate"
printf 'Run every remaining block from: %s\n' "$audit"
```

Run **each remaining block from the printed audit-copy root**, in a fresh Bash
shell (`bash --noprofile --norc`); no block relies on another block's variables
or working-directory changes. The clone's integration ref is deliberately
re-pointed at the candidate because the replay script reads that fixed ref.
This makes the replay self-referential: candidate code and evidence are compared
with expected bytes and a digest committed in that same candidate. It proves
internal agreement, not agreement with an independently trusted integration tip
or historical release. The supplied repository's integration ref is unchanged.

```sh
set -eu
q=qa/portfolio/agent-cost-accounting/pf-60-s03
rc=0
python3 -B "$q/acct-inventory-79/verify_acceptance.py" || rc=$?
printf 'Acceptance exit: %s (expected 2)\n' "$rc"
test "$rc" -eq 2
python3 -B "$q/acct-inventory-79/check_verifier.py"
python3 -B "$q/acct-selfcheck-84/check_current_tip.py"
```

**Read the outputs, including exit status.** The verifier must end with `BASELINE MATCH` and
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
[build setup](../../../../../docs/install.md#source-build). Before these blocks,
install Rust 1.95.0 (the pin in `codex-rs/rust-toolchain.toml`), `just`,
`cargo-nextest` and platform build tools described there. The first block runs
`cargo fetch --locked` while network access is available: this populates the
active Cargo home's registry and git dependency caches for the lockfile. On a
network-isolated host, transfer those caches from a trusted same-lockfile build
host first and replace that fetch with `cargo fetch --locked --offline`; record
that adaptation. Cargo's offline flag does not prevent build scripts fetching
external artifacts such as rusty_v8 archives; prepopulate those caches too for
a fully disconnected build. Never copy a Corbanu profile or credential files.

These blocks use the same dedicated target under `acct-operator-96/target/`,
covered by the committed `*/target/` rule; the first block prints that match.
Build prerequisites first. Then run each test block separately, inspecting its
exit/log before dispatching the next. Stop successors on any native credential
prompt or live-profile read; never authorize it.

```sh
set -eu
q=qa/portfolio/agent-cost-accounting/pf-60-s03
export CARGO_TARGET_DIR="$PWD/$q/acct-operator-96/target/cargo"
export NEXTEST_TEST_THREADS=4 CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0
export CARGO_PROFILE_DEV_DEBUG_ASSERTIONS=true CARGO_PROFILE_TEST_DEBUG_ASSERTIONS=true
git check-ignore -v "$CARGO_TARGET_DIR"
cat docs/development/test-isolation.md
cd codex-rs
cargo fetch --locked
cargo build --locked --offline -p codex-cli --bin codex -p codex-rmcp-client -p codex-code-mode-host --bins
```

```sh
set -eu
q=qa/portfolio/agent-cost-accounting/pf-60-s03
export CARGO_TARGET_DIR="$PWD/$q/acct-operator-96/target/cargo"
export NEXTEST_TEST_THREADS=4 CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0
export CARGO_PROFILE_DEV_DEBUG_ASSERTIONS=true CARGO_PROFILE_TEST_DEBUG_ASSERTIONS=true
cd codex-rs
just test -p codex-core accounting
```

```sh
set -eu
q=qa/portfolio/agent-cost-accounting/pf-60-s03
export CARGO_TARGET_DIR="$PWD/$q/acct-operator-96/target/cargo"
export NEXTEST_TEST_THREADS=4 CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0
export CARGO_PROFILE_DEV_DEBUG_ASSERTIONS=true CARGO_PROFILE_TEST_DEBUG_ASSERTIONS=true
cd codex-rs
just test -p codex-core accounting --features codex-core/developer-accounting
```

```sh
set -eu
q=qa/portfolio/agent-cost-accounting/pf-60-s03
export CARGO_TARGET_DIR="$PWD/$q/acct-operator-96/target/cargo"
export NEXTEST_TEST_THREADS=4 CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0
export CARGO_PROFILE_DEV_DEBUG_ASSERTIONS=true CARGO_PROFILE_TEST_DEBUG_ASSERTIONS=true
cd codex-rs
just test -p codex-tui usage
```

Require the exact banner
`Test isolation: disposable profile; native keyring disabled (debug lane)`
before every test lane, nonzero runs and zero failures/timeouts: the
recorded baseline is 124/124 default, 127/127 feature and 91/91 TUI (342 overlapping
executions), with 3545/3545/4078 skipped. Record fresh counts and failure names;
later passes never erase the retained round-66 timeout. Stop successors on any
native credential prompt or live-profile read; never authorize it.

**If replay disagrees:** preserve the candidate SHA, command, exit, receipt and
raw stdout/stderr before retrying. Read the acceptance exit table: 3 means drift
or refused baseline; 1/traceback is failed execution/check; missing history/ref is
a prerequisite failure. The comparison reference is
`qa/portfolio/agent-cost-accounting/pf-60-s03/acct-reference-88/expected.stdout.txt`,
with digest `25450ea3837dfc7624189eb89f7067fda71b76dbec21e2853122875f03dd1fce`
in the adjacent `reference.json`, read from the candidate commit. Compare the
retained `acceptance.stdout.txt` against that file; compare candidate evidence
files and hashes with those in the supplied candidate SHA recorded at clone time.
This is the same self-referential consistency check described above, not an
independent historical reference. Restore accidental edits or send intentional claim/evidence changes to the
integrator for review, refreshing inventories and expected output/digest together
only after justification. Never regenerate expectations merely to pass. Keep
scope-zero, settlement, independent package/platform/profile and live-repository
gaps open until their separate evidence and authority requirements are met.
