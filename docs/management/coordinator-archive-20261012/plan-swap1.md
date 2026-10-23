You are the plan-lifecycle worker. Read sec-common.md in /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/ (binding).
Travis decided: close the active plan docs/plans/active/p0-security-levels.md (slot 1) and activate
docs/plans/proposed/p1-security-hardening.md in its place. Follow the repo's plan lifecycle rules exactly
(docs/plans/check.py, docs/plans/tests, docs/sprints/check.py, PLAN_TEMPLATE.md, index files, docs/plans/completed/).
- Move P0 to completed with an honest closing note: all sprints merged/archived; PF-13-S07 final archive (waits on the
  Aggressive milestone) is carried forward into the P1 plan as an explicit open item (move/record it per the rules);
  link the Task Node rewards (P0SEC-TN-01..09) and the demo reel
  (https://corbanu-p0-security-reel.cloudflare-share.workers.dev/).
- Activate P1 in slot 1 with worktree coordinates as the checker requires, so PF-27-S06 (merged; gate pending a real
  Windows machine) can leave draft with the correct status, and PF-27-S07/S08/S09 (being created by worker sec-win2 in
  docs/sprints/current/p1-security-hardening/) are valid under an active plan. Do not edit sec-win2's sprint files
  except as strictly required by the plan move; coordinate by merging origin/main.
Both checkers must pass locally. One docs-only PR, merge when green (`gh pr merge N --merge --match-head-commit <sha>`).
Report concisely.
