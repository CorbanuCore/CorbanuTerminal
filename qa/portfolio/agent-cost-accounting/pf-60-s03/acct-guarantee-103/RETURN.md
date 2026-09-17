# RETURN

Action `acct-guarantee-103`; allocation digest
`4506481a41b013024771d6c4355cdacdeebdddb4b6a8058f02f844939dc57099`;
claim `182ab7a5-6c57-4cb6-a8c0-1b11f9f058eb`.
Worker: `gpt-6-astra`, `high`. ACK preceded START and tools.
Brief SHA-256 verified:
`8c187001e12ae116b15fcc0066856f967fe5e0e02425c7ec4a8bc6e211c693a5`.
Assigned and observed base: `2731b5e8868cb7222e4fedef3493ab977ecc79db`;
initial checkout clean.

The [round-97 RETURN](../acct-criterion-97/RETURN.md) now says:

> The frozen guard required an absolute, nonexistent destination that was not a
> symlink, and ran `git check-ignore -v` only when its literal spelling began
> with `$PWD/`. It did not resolve ancestry: an outside-spelled symlink alias into
> an unignored source directory could pass.

The unqualified “Inside-source destinations must be ignored” sentence is gone.
The record distinguishes the separate round-100 resolved-path change and its
segment evidence from round 97's frozen complete execution. Any additional
destination guarantee requires a separate integrator decision. This revision
does not change destination acceptance criteria, frozen notes or old execution
receipts.

The [pinned witness](../acct-final-100/test_destination.py) still loads the old
guard from `967425cc72ea28b5f59ecc38a56419b7db6885e8` and the new guard from
`a8dfff98892e60aaa0d7f05321fd7e57b79cdc2a`. It now prints the two subprocess
return codes and uses explicit failures, active under Python optimization.
[Observed stdout](witness-clean.stdout.txt) from the successful seven-case run:

```text
Observed alias-unignored-regression: old=0/new=1
```

The final JSON summary also takes its exit fields from those observations.
[Receipt](witness-clean.json) retains both commits, note/guard hashes, commands,
stdout/stderr and observed exits. The initial [pinned run](pinned-destination-results.json)
is retained separately. Reproduce from the repository root:

```sh
python3 -B qa/portfolio/agent-cost-accounting/pf-60-s03/acct-final-100/test_destination.py
```

An injected wrong old commit produces observed `old=1/new=1` and exit 1 in
both [normal](witness-mismatch.stdout.txt) and
[optimized](witness-mismatch-optimized.stdout.txt) execution. Their stderr files
record the explicit mismatch exception; neither writes a passing receipt.
These are deliberate negative controls, not failures of the production candidate.

Every subprocess in the witness, including `git show`, `git init` and both
shell guards, receives the same allowlisted environment as the round-102 checker:
only inherited PATH plus `GIT_CONFIG_GLOBAL=/dev/null`,
`GIT_CONFIG_NOSYSTEM=1`, `GIT_TERMINAL_PROMPT=0` and
`PYTHONDONTWRITEBYTECODE=1`; fixture shells additionally receive their PWD and
destination. It does not pass inherited Git/profile/shell variables.
[The ambient-environment control](witness-ambient-git.json) passed all seven
cases despite synthetic invalid Git dir/worktree/index/config overrides and a
BASH_ENV that would exit 97. This matches the prior checker's environment
boundary; it does not claim isolation from a replaced PATH executable or
repository-local configuration.

The four destination refusals now consistently print `STOP:` before refusing:

| Refusal | Message after STOP: | Exit |
| --- | --- | --- |
| Older Python | destination guard requires Python 3.9 or newer. | 1 |
| Existing destination | destination already exists; preserve it and choose a new path. | 1 |
| Destination symlink (including dangling) | destination is a symlink; preserve it and choose a new path. | 1 |
| Failed inside-source ignore check | inside-source destination did not pass git check-ignore; choose an ignored path. | Git's nonzero status (1 observed for unignored paths) |

The already explicit relative-path refusal remains. [Checks](checks.json) retain
nine live destination cases and two simulated Python-version branches, all with
expected exits and outputs. Version simulation is not qualification on actual
Python 3.8/3.9 runtimes. Only the destination segment ran; no new complete
six-block replay is claimed. Blocks 2–6 still equal the frozen round-97 blocks.
The live note's reference-digest prose was refreshed after these segment tests;
the final check compares the exercised segment with the final block bytes.

The [acceptance paragraph](../acct-acceptance-75/acceptance.md) attributes the
five adversarial reviews to the manager and names the recurring defect class:
**self-confirming evidence, where a check or claim appeared to validate more
than it tested**. It links the concrete catches and corrections: literal verdict
inspection; fresh-shell execution; dirty-source/wrong-root controls; a symlink
counterexample and prose/evidence comparison; and frozen/live byte plus
reproduction-input comparison. It names this follow-up's remaining defects.
The manager's count is not presented as an independently verified reviewer
roster or approval.

Adding that paragraph changed an inventoried document. Its inventory entry,
derived totals and correction history were refreshed without membership or
measurement changes. The exact-output reference changed only its acceptance
inventory totals line, derived from the new document size, and its digest:
`97316af06e636192bd1086eafae4bc7acb6c03b5faa95239adfd48096209db60`.
Old inventories elsewhere remain historical snapshots. The final verifier stdout
matches the amended reference byte-for-byte:
`RESULT agreement=20 disagreement=0 unavailable=3 exit=2`.
All 13 final verifier controls pass. Initial failures are preserved: the
unrefreshed inventory and new numbered link labels caused verifier exit 3;
the first control run consequently failed its clean-checkout prerequisite.
Descriptive labels replaced the unsupported round-label spellings without
weakening the numerical checker. Separate final streams preserve the corrected
runs. The unchanged three historical binaries remain unavailable.

Prerequisites built first, exit 0. Fresh Rust gates ran sequentially from
`codex-rs` using guarded `just test`, shared dedicated
`acct-activation-33/feature/target`, `NEXTEST_TEST_THREADS=4`, two build jobs
and debug assertions enabled. The isolation document was read before testing.

| Lane | Passed/run | Skipped | Failed | Timed out | Flaky | Leaky | Exit | Failure names |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `just test -p codex-core accounting` | 124/124 | 3545 | 0 | 0 | 0 | 0 | 0 | [] |
| Same, `--features codex-core/developer-accounting` | 127/127 | 3545 | 0 | 0 | 0 | 0 | 0 | [] |
| `just test -p codex-tui usage` | 91/91 | 4078 | 0 | 0 | 0 | 0 | 0 | [] |

[Raw logs and receipts](gates-01/) bind exact commands and counts. All lanes
emitted the disposable-profile/native-keyring-disabled banner. No native
credential prompt or live-profile read was observed. No failed Rust lane was
retried. The auxiliary-scope test passed in 31.531s default and 31.551s feature;
these passes do not resolve its historical timeout. The total is 342 overlapping
test executions, not distinct tests.

Changed existing files: **+101/-32 across eight files**; [exact hunks](changed-lines.patch).
[Scope accounting](scope.json) also lists new helpers and evidence with line/byte
counts and hashes, excluding itself. [Final checks](final-check.json) verify
scope, whitespace, Python syntax, exercised guard bytes, witness observations,
reference equality and gate log hashes/counts. All changes are within the
assigned QA subtree. No Rust/product edit, workspace formatter, source commit
or push occurred.

Brief precision: the old witness already ran both commits and asserted their
exits; its final printed summary was literal. Thus “prints its expected result
proves nothing” overstates the defect in the whole witness, though the reporting
defect was real and is fixed. The three silent refusals are existing path,
symlink and ignore-check failure; the relative-path refusal already had STOP.
The supplied five-review count and sixth-refusal characterization are manager
context; this worker has no independent review roster to certify either.
No other factual error was found in the requested corrections or Rust gate.

Classification: routine internal evidence correction. Product heading
**Measurement targets**, excerpt “No commercial performance numbers have been
supplied.” Used the `corbanu-terminal-development` skill. No plan/sprint
lifecycle change. True-TUI, code-blind functional, live-repository and benchmark
qualification are N/A to this internal revision, which changes no product
workflow and makes no functional handoff; the S03 functional gate remains open,
with integrator acceptance of N/A left to Fable. No owner approval, S03 acceptance,
unqualified human-test readiness or release qualification is claimed.
