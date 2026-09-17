# RETURN

Allocation `acct-selfcheck-84`; model `gpt-6-astra`; effort `high`.
Claim `902201fb-4784-4896-9a87-0d376b9cb94d`.
Allocation digest `35373d89cbe965bad6c44547a31dae5e64263d66f9e6342b37c1dc3c9b7e38f9`.
Brief SHA-256 verified:
`f98184b7c132aa5b4a2fdd1c37dc6f6331aadf7a015b0d5f195f369d30cb5503`.
Starting and finishing integration HEAD:
`7b3173115b015ef36c33dc2a8bfd081566cd23d2`,
branch `bootstrap/acct-activation-20260916`, assigned worktree.

Routine internal evidence maintenance under PF-60 / PF-60-S03, not a product
behavior change. Product heading **Product measurement**, subsection
**Measurement targets**, excerpt “No commercial performance numbers have been
supplied.” Independent functional design/execution, true-TUI, live-repository
and benchmark qualification are not applicable to this evidence-tool revision;
final-package independent functional qualification remains due for S03.
The manager owns canonical sprint/plan records and acceptance of this N/A.
No human acceptance, review approval, S03 completion or release is claimed.

## Documented baseline is enforced

[verify_acceptance.py](../acct-inventory-79/verify_acceptance.py) reads the fenced
baseline and unavailable-item list in
[acceptance.md](../acct-acceptance-75/acceptance.md). It compares the derived
agreement/disagreement/unavailable counts, evidence exit and exact unavailable
identities. It rejects absent/ambiguous baseline clauses, duplicate/malformed
items, inconsistent list/count or exit, and any baseline purporting to permit
disagreements. It adds a separate BASELINE line without inflating agreement counts.

- **Exit 2 / BASELINE MATCH:** retained baseline is still exactly
  **20 agreements, 0 disagreements, 3 unavailable**, naming the committed
  `codex`, `codex-code-mode-host`, and `rmcp_test_server` package artifacts.
- **Exit 3 / BASELINE DRIFT:** the retained result differs from that document,
  including an extra absent artifact or an invalid baseline. A mismatch is
  distinct from the deliberately incomplete package baseline.
- Optional local-package checks run after the retained comparison. A local
  disagreement returns **1**, unless retained drift takes precedence. Missing
  local package files still return **2** if the retained baseline matches.
  **0** is reserved for complete agreement with a matching complete baseline.

[The revised controls](proposed-controls/verifier-checks.json) pass **11/11** in
a clean proposed-snapshot clone. Existing wrong-claim and corruption cases now
return 3. Removing the gate log yields **18 / 0 / 5, exit 3**. Count and
identity mutations refresh the document inventory first, leaving actual retained
counts at **20 / 0 / 3**; this proves the baseline check itself catches them.
Missing baseline, wrong baseline exit, duplicate names and optional local-file
disagreement are also covered. Raw stdout/stderr are retained next to the receipt.
No binary was launched; the local-file mutation used synthetic text.

The proposed snapshot is `d4d3c0ffd78da62b2cd027b8ac75a2ecb48a750a` in a disposable
clone, not an integration commit. [Final-tree checks](final-tree-check.json)
compare the tested scripts, acceptance/inventory files and disclosures byte for
byte against the final working tree and reproduce the proposed output.

## Exclusion attribution and remaining prose coverage

[The exclusion record](../acct-baseline-81/package-exclusion.md) now states that
retaining the exclusion is this acct-selfcheck-84 Astra High worker's engineering
judgement, dated September 17, 2026. The original decider/time are not established
by the retained evidence, and no decision is attributed to Travis or Fable.
Reason: avoid committing **713,231,616 bytes / 680.191 MiB** of unsigned local
debug outputs while retaining manifest/build evidence. This does not qualify
missing bytes or waive final-package acceptance. Accessible immutable storage of
the exact historical package would close availability; a rebuild alone would not.

[The worded-quantity register](worded-quantity-residual.md) identifies unchecked
sentences/regions by their openings, including “most”, “only”, “none”, absence,
completeness and causal/status claims. Only the three existing literal worded
count patterns have direct bindings; arbitrary new worded claims are not caught.
The baseline block and unavailable identities now have their own check.

Closing prose coverage requires a typed claim registry with units/scopes and
evidence bindings, generated measurable sentences, and review enforcement for
new/changed qualitative claims. My judgement: worthwhile for a maintained report
generator; an unrestricted English quantity parser is not worthwhile in this
bounded revision. The remaining gap is disclosed, not waived.

## Current integration tip reproduces

[The untouched-tip receipt](current-tip/receipt.json) records a fresh full local
shared clone detached at **7b3173115b015ef36c33dc2a8bfd081566cd23d2**, resolved
from HEAD at execution time. No working-tree edits or ignored packages were
copied. Git status was empty before and after; the source tip stayed unchanged.

The full default verifier returned **20 / 0 / 3, exit 2**, stderr empty.
Stdout is byte-identical to the round-81 proposed replay, SHA-256
`cd5951e7f00cc4307326ec89d1968ea8c4958aa88d08a9606b9015fc4e7f8807`.
The original current-tip harness also passed **5/5** controls, including the
local-package finalizer; raw outputs are in [current-tip/controls](current-tip/controls/).

Thus there is **no observed decay at the current integration tip**. This does
not guarantee future tips or equate expected package absence with complete
qualification. The new drift exit detects future divergence. Historical hashes
and base references describe retained evidence; they do not require executing
only at the original producer commit.

`check_current_tip.py` resolves HEAD anew and allocates a fresh clone each run.
It preserves attempts instead of overwriting the earlier hard-coded replay.
These are worker-run evidence checks, not independent functional acceptance.

## Required fresh Rust gates

Prerequisites built first from `codex-rs` with shared dedicated
`acct-activation-33/feature/target`, `NEXTEST_TEST_THREADS=4`, and debug
assertions enabled. All tests used guarded `just test` after reading
`docs/development/test-isolation.md`.

| Lane | Passed/run | Skipped | Failed | Timed out | Flaky | Leaky | Exit |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `just test -p codex-core accounting` | 124/124 | 3545 | 0 | 0 | 0 | 0 | 0 |
| Same with `--features codex-core/developer-accounting` | 127/127 | 3545 | 0 | 0 | 0 | 0 | 0 |
| `just test -p codex-tui usage` | 91/91 | 4078 | 0 | 0 | 0 | 0 | 0 |

**342/342 executions**, overlapping filters; exact failure names: **none**.
The historical auxiliary-scope test passed at **32.872s** default and
**32.292s** feature. This does not resolve its prior unexplained timeouts.
[Test results](test-results.json) and `gates-01/` retain commands, settings,
raw compressed logs, hashes and counts. The isolation banners were present;
no native credential prompt or live-profile access was observed.
Python syntax and `git diff --check` passed. No Rust/product source changed,
and no workspace-wide formatter was run.

## Changed lines and brief corrections

Six existing files: **+135/-25**:
acceptance **+18/-7**, acceptance inventory **+7/-7**, exclusion **+14/-0**,
control harness **+37/-3**, inventory correction **+19/-5**, verifier **+40/-3**.
[Scope receipt](scope.json) separates new scripts, documentation and raw evidence,
with per-file sizes, line counts and hashes; it excludes itself to avoid recursion.
The previous inventory correction remains preserved alongside this refresh.
All writes, caches and disposable clones stay within the assigned QA subtree.
No source-worktree commit or push was performed.

The substantive findings are supported. “Last uncovered class” is too broad:
other excluded code/link/identifier regions and qualitative truth/authorization
claims remain outside general numeric reconciliation. The dedicated baseline
check closes only its explicitly documented region. The brief's reviewed/received
commit `ecfd6174c` and submission to Travis are manager-supplied provenance,
not independently verified status; this checkout's actual current tip is the
full hash above. No acceptance decision is fabricated.
