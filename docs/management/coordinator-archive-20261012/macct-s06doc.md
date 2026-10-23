You are a docs worker for PF-84 (named accounts per provider). Read
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md (binding) and macct-lane2.result.md in that folder
("For Travis to decide" item 1), the corbanu-terminal-development skill (sprint/plan procedures), and on origin/main
docs/sprints/SPRINT_TEMPLATE.md, docs/plans/active/unified-provider-auth.md (PF-84 section, where PR #423 recorded the
ChatGPT/AWS question) and the PF-84-S04/S05 records.
Travis decided on 2026-10-11 (chat with the coordinator): option (a) — a new sprint PF-84-S06 after S05: named AWS profile
accounts first, then named ChatGPT/OpenAI sign-in accounts. Do, in one docs-only PR:
- Record the decision (dated, attributed) in the plan's PF-84 section, replacing the open question.
- Create docs/sprints/current/unified-provider-auth/pf-84-s06-<slug>.md as a DRAFT (unallocated) from the template, depending
  on PF-84-S05, sliced into slice A (AWS profile accounts: non-secret profile name, Bedrock's `aws.profile`, refusal/never
  fall back, how to verify live without AWS credentials — state it as NOT VERIFIABLE unless a test AWS profile is provided,
  and list what Travis would need to provide) and slice B (ChatGPT sign-in accounts: per-account sign-in store in the vault,
  refreshed tokens saved back to the right account, login flow aimed at an account, migration of today's single `Codex Auth`
  item/auth.json to the `default` account, security notes). Concrete acceptance criteria, gates matching the other PF-84
  sprints (GLM 5.3 Flash runs, tmux, videos, Opus 5.5 High review), and the open points Travis may need to decide.
- Update the sprint index/plan tables and navigation as the checkers require.
Run `python3 docs/plans/check.py && python3 docs/sprints/check.py`; they must pass. Another worker (macct-lane3) is editing
S04/S05 code and records concurrently — touch only S06's record and the plan/index lines for S06 and the decision; merge
origin/main on conflict. Merge only when ALL checks are green, never with checks pending (`gh pr merge N --merge
--match-head-commit <sha>`; never force-push; never rm -f). Report: PR number, checker output.
