# PF-60-S01 kickoff and handoff

## Native Anthropic subtotal follow-up — 2026-09-11

Frozen UNCOMMITTED/unstaged patch based on clean HEAD
`58b214a0ab1f220a5a768dd605697f7bae54c0ba` in the existing worker worktree
`/Volumes/CorbanuDrive/Corbanu/worktrees/accounting-pf60-s01-20260911`, branch
`workstream/accounting-pf60-s01-20260911`. The already committed/reviewed
partial-cache validation fix is preserved. This receipt supersedes the previous
candidate receipts below, retained as historical evidence.

Class: bounded same-owner fix in active PF-60 / PF-60-S01 (`in_progress`), under
the parent's supplied standing authority. Product citation: **Product
measurement** — “No commercial performance numbers have been supplied.” No new
sprint, contract decision or dispatch approval is required for this correction.
The development skill routed the work through existing rules and local checks;
the user's four-file scope governs evidence updates and leaves shared ledgers
with the parent.

Native Anthropic noncached input is now an internal pricing measurement even
when inclusive input cannot be computed. With `child-a-1` cache creation absent,
known cost is `(50 * 3 + 10 * 0.3) / 1,000,000 = 0.000153 USD`, previously
`0.000003`. Inclusive input, write, output, total and full cost remain unknown.
The private count is excluded from rows/checkpoints and cannot be double counted
with known cache. Chat/Responses inclusive dialects and unknown compatible
dialects retain their existing handling. No runtime/public schema changes.

Seven regression methods cover the reported case and root subtotal, all 54
omitted/null/zero input-bucket matrix subcases, missing prices, explicit zero,
partial JSON replay with replacements and 24 reordered/duplicate permutations,
wire isolation, raw-count validation and unchanged row/checkpoint shape.
Baseline: 41 passing tests. Failing before: 48 tests, 27 failures across five new
methods, no errors. Passed after: all 48 tests. Plans: 3/3 active; sprints:
115 current / 121 archived. Local whitespace check passed. Exact commands,
arithmetic and code/input hashes are in [results.md](results.md); the frozen
full-index patch hash and all four changed-file hashes accompany delivery.

Only `reference.py`, `test_contract.py`, `results.md` and `handoff.md` changed.
Contract/README/fixtures and all 41 prior tests are preserved. No additional
agents, reviewers or model calls were made. Parent independently reviews this
uncommitted patch, verifies the final hashes and recomputes partial arithmetic,
then reruns fixture/governance checks on its combined tree before any local
scoped commit. No independent-review or combined-tree pass is claimed here.

No source cutover, shared-plan edits, network/private-log/credential access,
runtime/schema/retention/price-policy/live-bill changes, push/merge/release or
S02 activation occurred. Contract/human acceptance, native/TUI/live-repository
qualification, benchmark and S02+ gates remain unchanged and open.

## Follow-up correction handoff — 2026-09-11

Frozen uncommitted correction based on clean HEAD
`1a29602e8ab5605ed781dd5ddf1eb49e6033b872` in the worker coordinates below.
This receipt supersedes the original test/oracle candidate, not its contract or
human gates. The original kickoff receipt follows as historical evidence.

`normalize` now rejects a known read/write subtotal above known inclusive input,
including Chat/Responses input 10, read 20, write unknown and Responses input 10,
write 20, read unknown. Missing measurements remain unknown. Eight added methods
cover invalid/valid partial splits, complete-sum boundaries, unknown input,
Anthropic dialect semantics and per-revision replay. Baseline: 33 passing tests;
regressions before fix: 41 tests with 13 failing subcases; corrected: 41 passing.
Local governance: 3 active plans, 115 current / 121 archived sprints; the parent
reports 116 current in its manager tree. Exact commands/digests: [results.md](results.md).

Changed files only: `reference.py` (+3/-2 non-test source lines),
`test_contract.py` (+102/-0 test lines), `results.md`, `handoff.md`, all under this
QA directory. No staging or commit; parent owns mandatory independent structured
review, combined-tree verification and any scoped local commit. No follow-up
agents/reviewers/model calls were launched. All runtime, S02, human acceptance
and release gates remain open; this is synthetic fixture evidence only.

## Original kickoff handoff (historical)

Delivered scope: proposed accounting contract v1 plus hand-computed synthetic
goldens and executable reference tests. Product initiative, feature PF-60,
sprint PF-60-S01 allocated `in_progress`; worker output does not complete or
archive it. Canonical product citation: **Product measurement** — “No commercial
performance numbers have been supplied.” See the
[contract](../../../../docs/research/agent-cost-accounting/contract.md).

| Receipt field | Value |
| --- | --- |
| Worker | Explicitly requested Astra High accounting lane; no additional agents or reviewers |
| Worktree | `/Volumes/CorbanuDrive/Corbanu/worktrees/accounting-pf60-s01-20260911` |
| Branch | `workstream/accounting-pf60-s01-20260911` |
| Verified launch / source base | `f17e5a54daba11c6da2554f14b62f11ace752142`; clean at launch |
| Allocated ancestor | `295aed26e53b17f919f7199ae1c9748b1b1250ba`; launch authorizes fast-forward, manager owns ledger reconciliation |
| Candidate | PF-60-S01 proposal v1; five tested artifact SHA-256 values in [results.md](results.md); exact containing worker commit in delivery message |
| Rules read | Root `AGENTS.md`, corbanu-terminal-development skill, plan/sprint indexes, active PF-60 plan, allocated S01 and canonical citation; no nested AGENTS under output paths |
| Integration owner | Codex management; Travis Good accepts contract/human decisions |

Changed files (all new; no unrelated files overwritten):

- `docs/research/agent-cost-accounting/contract.md`: native mappings, identity,
  replay/unknown semantics, proposed vocabulary, precision, retention and prices.
- `qa/portfolio/agent-cost-accounting/pf-60-s01/fixtures.json`: synthetic source
  observations, provider prices, hierarchy, replay order and literal expectations.
- `qa/portfolio/agent-cost-accounting/pf-60-s01/reference.py`: isolated Python
  contract oracle; no native runtime registration or new production authority.
- `qa/portfolio/agent-cost-accounting/pf-60-s01/test_contract.py`: 33 tests.
- `qa/portfolio/agent-cost-accounting/pf-60-s01/README.md`: explicit arithmetic,
  commands, negative cases and test applicability limits.
- `qa/portfolio/agent-cost-accounting/pf-60-s01/results.md`: actual results,
  candidate digests and external CI blocker provenance.
- `qa/portfolio/agent-cost-accounting/pf-60-s01/handoff.md`: this receipt.

Local evidence: exact unittest discovery passed 33 tests; plan and sprint
governance checks passed. See results for timestamps, actual outputs and negative
assertions. Final staged scope/whitespace checks passed at 09:57:14Z; exactly seven
new files are in the permitted scope. Exact commit accompanies the delivery
receipt. No push, main merge, plan/sprint ledger edits, PF13 changes,
runtime Rust/migrations, manifests, CI/workflow changes, live dashboard writes,
network writes, paid data, feature enablement or release is part of this output.

The source trace establishes why runtime follow-up is needed: throttle state
overwrites per provider/model/key; raw per-response usage is transient; persisted
TokenCount is cumulative and may contain synthetic context values; native
defaults lose presence. Corbanu API balance is gateway money/reservation state,
separate from provider allowances and task estimates. Its compatibility route
uses Chat wire and cannot supply measured cache-write zero. Historical exact
request prices, invoices and lost usage presence cannot be reconstructed honestly
from the inspected seams alone.

Next human decisions: Travis accepts or revises the measured/estimated/billed/
allowance/balance vocabulary, 90-day detail / 365-day aggregate retention proposal,
replay horizon/tombstones and deletion behavior, exact USD arithmetic/display
precision, effective-dated price authority and historical unknown policy. Actual
price source, billing allocation and data retention are not approved by fixtures.

Next manager actions: independently recompute goldens and check source mappings,
review only the scoped diff, rerun fixture and governance commands on the combined
tree, obtain Travis's acceptance, and update/archive S01 in the shared ledgers
only when its exit evidence is met. Remote CI remains externally blocked by the
manager-reported account billing lock; no workflow repair is warranted here.

S02+ remain blocked. Before S02: accepted/archived S01, verified upstream SHA,
serialized native schema and exact adapter allocation, durable dispatch/retry
identity and presence capture, native restart/replay/cancellation tests and
historical-import rules. S03 depends on S02 and full cache-write/unknown display;
S04 depends on S03 with a recorded binary, actual interactive and applicable live
repository flows plus named-human acceptance. UI, benchmarks, runtime proof and
release readiness are pending/not applicable to this fixture-only preparation.
