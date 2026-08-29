# Security sprint refactor — planning evidence

- Decision: Travis Good, 2026-08-27, approved the proposed scope/dependency
  refactor, separately scoped browser isolation, and bounded sprint concurrency.
- Classification: approved product-initiative planning; routine process/checker
  implementation. No product runtime implementation or release acceptance.
- Product linkage: **P0 `/security` levels**; “Permissive preserves the shipping
  behavior and does not silently change existing policies.” Expanded linkage:
  **Moderate/Aggressive isolation and content provenance**; “Browser isolation
  is a separately scoped feature within the security initiative.”
- Plan: `docs/plans/active/p0-security-levels.md`.
- Worktree: `/Users/travisgood/Documents/ChatGPT/corbanu-pf13-s02`.
- Branch: `feat/pf-13-s02-scoped-vault-resolver`.
- Base: `549c18f0b63b8e5c4fedf60b18932d1d48adb56f`; change is the commit containing
  this record. No completed sprint was reopened or new sprint activated.

## Scope and scheduling

PF-27 owns shared contracts, PF-28 confidentiality, PF-29 source/derived taint,
and PF-30 browser isolation. PF-26-S01 now constructs early harnesses;
PF-26-S04 qualifies the integrated automated candidate before S02 true-TUI and
S03 human/docs/release acceptance. The active plan opts into three independent
slots; blocked work retains a slot, write-scope collisions remain forbidden,
and future worktrees are unallocated until dependency-complete readiness.

## Verification

| Check | Result |
| --- | --- |
| `ruff format --check docs/plans/check.py docs/plans/tests/test_check.py docs/sprints/check.py docs/sprints/tests/test_check.py` | pass |
| `ruff check` on those four Python files | pass |
| `python3 -m unittest discover -s docs/plans/tests -p 'test_*.py'` | 4 pass |
| `python3 -m unittest discover -s docs/sprints/tests -p 'test_*.py'` | 19 pass |
| `python3 docs/plans/check.py` | pass; one active plan |
| `python3 docs/sprints/check.py` | pass; 25 current, 84 archived; only PF-13-S05 active |
| `python3 scripts/check_portable_skills.py` | pass; 25 mirrored files unchanged |
| `python3 -m unittest discover -s scripts -p 'test_check_portable_skills.py'` | 3 pass |
| `uv run --with-requirements requirements-docs.txt mkdocs build --strict --site-dir <temporary-directory>` | pass; rendered navigation includes every new sprint |
| `git diff --check` | pass |

Structured closeout review used
`python3 /Users/travisgood/.codex/skills/autoreview/scripts/autoreview --mode local --engine codex --no-web-search --stream-engine-output --prompt <approved-refactor-scope>`.
The helper returned exit 0 with no findings: “patch is correct.” This reviews
the process/planning patch, not PF-13's outstanding independent security audit.
The documentation build initially caught a research link outside the docs tree;
it now uses the pinned repository artifact and strict validation passes.

## Unchanged qualification status

PF-13-S05 remains in progress: the 135 full-Core failures require triage and a
clean rerun, separate-machine Windows follow-up remains due, and independent
security review is pending. Prior canary success is not full certification.
No Rust suite was rerun for this documentation/Python-only change. New-feature
TUI, live-repository, human acceptance, and due benchmark/release evidence remain
required in their owning sprints; this record does not satisfy those gates.
