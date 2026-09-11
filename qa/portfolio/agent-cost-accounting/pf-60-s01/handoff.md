# PF-60-S01 kickoff and handoff

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
