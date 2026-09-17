# RETURN

Allocation `acct-inventory-79`; model `gpt-6-astra`; effort `high`.
Claim `5429d53f-c91a-4705-8b00-986f1153a557`.
Allocation digest `48ebbdcdccbce4eca86e76d5e775d62dac14ffa31c3afba8fcbb78277d36111d`.
Brief SHA-256 matched `af66a81bde4d0c7e57f88cbc7a3b2561e53834f3413dfe36d89fcea87f9c094a`.
Starting HEAD matched `8317d81a49aac1246120e806b7e380c13c738862`.

Routine evidence correction. Product heading **Product measurement**, excerpt
“No commercial performance numbers have been supplied.” PF-60 / PF-60-S03 remain
active / in_progress; this revision does not qualify or accept S03. Runtime,
financial boundaries, release status and owner authorization are unchanged.
Independent functional/TUI/live-repository testing is not applicable to this
evidence-tool revision; the independent final-package functional gate remains due
for the affected product workflow. No named acceptance, release or benchmark pass
is claimed.

## Corrected inventory

[Round-75 scope](../acct-acceptance-75/scope.json) now classifies each path against
its original base: **15 new files**, **3 modified files**. Current full-file
totals are **47,679 new bytes / 632 new text lines**, **16,713 modified bytes /
230 modified text lines**; combined **64,392 bytes / 862 text lines**.
These are full-file sizes, not diff additions or deletions. The three modified
paths are round-72 `RETURN.md`, `acceptance-gap.md` and `summarize.py`.
The current totals include the new command paragraph in the acceptance document.
[inventory-correction.json](inventory-correction.json) preserves the previous
inventory and supersedes its totals and digest. Original round-76 scope/delta
remain historical snapshots, not current-tree checks.

## Clean-checkout command and evidence validator

From the repository root:

```sh
python3 -B qa/portfolio/agent-cost-accounting/pf-60-s03/acct-inventory-79/verify_acceptance.py
```

[acceptance-output.txt](acceptance-output.txt) is its complete verbatim output
today. It is byte-identical to [clean-checkout.stdout.txt](clean-checkout.stdout.txt):
**20 agreements, 0 disagreements, 3 unavailable; exit 2**.
All acceptance quantities, referenced gate results/timings, mutation counts,
page/viewport counts and refreshed inventory totals were re-derived.
Only the ignored package binaries cannot be re-derived from committed evidence:
`codex` **607,785,336 bytes**, `codex-code-mode-host` **93,937,576 bytes**,
`rmcp_test_server` **11,508,704 bytes**, and their claimed mode `0555` and SHA-256.
The retained manifest/build log do agree. Missing bytes are never a passed package.

[The previous validator](../acct-fitness-76/finalize.py) now delegates read-only
validation instead of rewriting receipts or crashing on missing package files.
It additionally verifies any locally present package bytes: on this producer
tree all three sizes, hashes and modes agree (23 agreements, 3 unavailable from
committed evidence, exit 2). On a clean checkout it reports both committed and
local absence for each binary (20 agreements, 6 unavailable, exit 2), continues
all other checks, and never launches a binary.

[verifier-checks.json](verifier-checks.json) records a clean sparse Git checkout
of the proposed QA snapshot, with no ignored package or build state copied.
Five controls passed: clean command, clean legacy entry point, changed numerical
claim, corrupted viewport, and missing gate log. Wrong/corrupt evidence returned
exit 1; missing evidence returned exit 2 while later items were still checked.
Raw stdout/stderr for every control is preserved. The temporary local snapshot
commit lives only in the disposable clone under ignored `target/`; no commit
or branch change was made in the source worktree. No network, live profile or
credentials were used. The checker was implemented and executed by this worker;
this is not an independent product-acceptance review.

## Fresh required Rust gates

Prerequisites built first, from `codex-rs`, shared dedicated
`acct-activation-33/feature/target`, `NEXTEST_TEST_THREADS=4`.
Only guarded `just test` was used, after reading test-isolation guidance.

| Command | Passed/run | Skipped | Failed / timed out / flaky / leaky | Exit |
| --- | ---: | ---: | --- | ---: |
| `just test -p codex-core accounting` | 124/124 | 3545 | 0 / 0 / 0 / 0 | 0 |
| Same with `--features codex-core/developer-accounting` | 127/127 | 3545 | 0 / 0 / 0 / 0 | 0 |
| `just test -p codex-tui usage` | 91/91 | 4078 | 0 / 0 / 0 / 0 | 0 |

**342/342 executions**, overlapping filters; failure names: **none**.
[test-results.json](test-results.json) and `gates-01/` preserve commands,
raw compressed logs, SHA-256 and parsed counts. No native credential prompt or
live-profile access was observed. These are fresh round-79 runs; the acceptance
command intentionally checks the round-76 receipt actually linked in the document.
Historical round-66 timeout failures remain open and are reported explicitly.

## Changed lines and brief precision

Existing files: **+77/-82** (acceptance +15/-0, scope +50/-24,
legacy validator +12/-58). [scope.json](scope.json) records every changed path,
its size/hash and exact diff numstat, excluding itself to avoid self-reference.
New helper/receipt/output lines are disclosed separately there.
Python parse and whitespace checks passed. Changes remain entirely within the
assigned QA subtree. No workspace-wide formatter or push was run.

Both P3 diagnoses were correct. “Cannot pass on a clean checkout” described the
previous validator accurately; the replacement deliberately returns an incomplete
status when package evidence is unavailable, not a fabricated all-pass result.
No numerical disagreement in the retained acceptance evidence remains.
The brief's reviewed/received status is manager-supplied authority, not a new
review or approval performed by this worker.
