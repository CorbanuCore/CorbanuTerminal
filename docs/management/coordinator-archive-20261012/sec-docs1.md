You are a security plan bookkeeping worker. Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md
first (working rules apply). Docs-only change, one PR to main.

Travis's decisions (2026-10-06):
- PF-27-S02 open decision "unsandboxed routes" (MCP servers, hooks, `!` shell, app-server command/exec in protected
  levels): ALLOW them as user-configured/user-started and SHOW them as "not contained" in the PF-41-S01 inspector.
  Add that display requirement explicitly to PF-41-S01's Remaining list.
- PF-27-S02 open decision "Claude panes / external provider tools": handed to the TUI lane (a worker is on it now);
  record it as a TUI-lane item (new bounded item in the PF-24-S02 or PF-25 record, whichever fits; say which).
- PF-27-S06 (Windows broker and launch) moves to the P1 hardening plan (docs/plans/proposed/p1-security-hardening.md and
  docs/sprints/current/p1-security-hardening/), dependencies intact. PF-27-S02's Windows verification item points there.
- Nested `corbanu exec` under Aggressive: configurable between refuse and pass-Aggressive-down, default refuse (a TUI
  worker is implementing it; just record the decision in the PF-24 record that owns Aggressive follow-ups).

Then, where the per-sprint gate evidence is complete and the only blocker was those decisions, mark completed and
archive per the repo's archive rules: PF-27-S02, then PF-28-S01 and PF-33-S01 if their checkers now allow it. Do not
archive anything whose gate evidence is incomplete; say what is missing instead. Keep the index, plan front matter and
mkdocs links consistent; both checkers (docs/plans/check.py, docs/sprints/check.py) must pass. Other lane workers edit
these files concurrently: merge origin/main right before pushing and keep the diff minimal. Merge when green.
Report: PR, what was archived, what was moved, anything left open.
