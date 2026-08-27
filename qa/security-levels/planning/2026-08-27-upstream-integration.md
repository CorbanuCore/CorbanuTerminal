# Upstream integration and parallel-lane planning evidence

- Authority: Travis Good, 2026-08-27, accepted the upstream/injection/autoreview
  recommendations, requested browser isolation as parallel work, and clarified
  that reconnect incidents occur in tmux on remote Linux machines.
- Classification: approved product-initiative planning and routine process/
  documentation work. No runtime, checker, dependency, or release changes.
- Product linkage: **Product principles**, “Maintain continuous Codex parity
  without removing Corbanu-specific behavior”; **P0 `/security` levels**,
  “Permissive preserves the shipping behavior and does not silently change
  existing policies”; **Moderate/Aggressive isolation and content provenance**,
  “Browser isolation is a separately scoped feature within the security initiative.”
- Worktree: `/Users/travisgood/Documents/ChatGPT/corbanu-pf13-s02`.
- Branch: `feat/pf-13-s02-scoped-vault-resolver`.
- Inspected base: `12bf62444bcab7c5eea6d25b23aa301993fcb0ab`; this is a fork
  commit, not a verified upstream baseline. Evidence describes the working-tree
  documentation change based on that commit. Travis Good subsequently authorized
  committing and pushing this documentation change to the same branch.

## Decisions reflected

- Root policy 1.4 links one canonical upstream integration process. Plan/sprint
  templates require baseline, file ownership, thin adapter boundaries, contract
  tests, and upgrade disposition. All 25 current sprints link their plan records.
- PF-27 owns shared integration seams before consumers. PF-30-S01 is explicitly
  prioritized alongside PF-29 and eligible PF-28 under the three-slot cap;
  PF-30-S02 is the later facade/content join. No sprint was activated and no
  hypothetical worktree was recorded as allocated.
- Injection coverage includes concrete ingress inventory, unsupported-path
  denial, pinned OpenClaw design provenance, forced detector misses, benign
  controls, and separate task-integrity versus confidentiality/authority results.
- PF-14's seven drafts retain native lifecycle and exact host-side routing,
  record security dependencies, and require current execution metadata and
  applicable per-sprint interactive proof. The plan remains proposed.
- The reconnect report is recorded separately in
  `qa/reliability/2026-08-27-linux-tmux-reconnect.md`, distinguishing attachment,
  process lifetime, and provider streaming. Root cause and remote versions are
  unknown; no timeout change, SSH configuration change, or fix is claimed.

## Verification

| Check | Result |
| --- | --- |
| `python3 docs/plans/check.py` | pass; one active plan of two |
| `python3 docs/sprints/check.py` | pass; 25 current, 84 archived; PF-13-S05 remains the only active sprint |
| `python3 -m unittest discover -s docs/plans/tests -p 'test_*.py'` | 4 pass |
| `python3 -m unittest discover -s docs/sprints/tests -p 'test_*.py'` | 19 pass |
| `python3 -m unittest discover -s scripts -p 'test_check_portable_skills.py'` | 3 pass |
| `python3 scripts/check_portable_skills.py` | pass; 25 mirrored files unchanged |
| `uv run --with-requirements requirements-docs.txt mkdocs build --strict --site-dir <temporary-directory>` | pass; new contract and sprint links resolve; existing excluded-archive notices are informational |
| `git diff --check` | pass |

Reviewed the final scope, cross-plan dependency graph, new source/adapter
requirements, sprint line limits, and requested versus actual activation state.
Structural validation is not proof of upstream ancestry or runtime compatibility.

Pre-commit closeout repeated all 26 process tests, validators, skill parity, and
the strict documentation build successfully. Structured review used
`python3 /Users/travisgood/.codex/skills/autoreview/scripts/autoreview --mode local --engine codex --no-web-search --stream-engine-output`
with a prompt restricting review to this approved documentation/process scope.
It exited 0 with no findings and “patch is correct.” No findings were rejected
or review-triggered fixes required. This is not PF-13's independent security
audit or proof of runtime/upstream qualification.

## Unchanged qualification and release status

Verified upstream ancestry and candidate upgrade evidence remain unresolved;
affected new implementation cannot become ready without its required record.
PF-13-S05 still needs triage/clean rerun of 135 Core failures, separate-machine
Windows follow-up, and independent security review. No Rust tests, true-TUI,
live-repository flows, human acceptance, or benchmark campaign were rerun for
this documentation-only change. Existing release gates remain in force; no
security feature, reconnect repair, or upstream upgrade is certified here.
