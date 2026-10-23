You are the planning worker for "named accounts per provider". Read multiacct1.result.md and sec-common.md in
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/, docs/plans/PLAN_TEMPLATE.md, the sprint format used in
docs/sprints/, and the proposed plan docs/plans/proposed/unified-provider-auth.md.
Travis's requirements: support MULTIPLE ACCOUNTS FOR ANY PROVIDER, covering BOTH alternate API keys (several keys for the
same provider, e.g. two Anthropic API keys, two OpenRouter keys, two Z.AI keys) AND alternate subscription/plan logins
(e.g. two Claude Plan accounts, two ChatGPT logins); choose the account per session, per worker/subagent, and from the CLI
(`corbanu exec`), and attribute usage/accounting per account. Workers spawned by the coordinator must be able to use a
different account than the coordinator.
Deliver planning only (no code): add a feature section to the proposed unified-provider-auth plan (or a new proposed plan
if that fits the rules better) and 3-5 draft sprint records in docs/sprints/current/<plan-dir>/ with acceptance criteria,
code boundaries (cite file:line from multiacct1), test plan incl. tmux SOP + GLM run + videos, security notes (vault labels,
keychain items, no secret leakage, per-account isolation), migration of existing single-account setups, and the
wrapper-script pitfall (`~/.local/bin/corbanu` resets CORBANU_HOME) as a fix item. Also include the duplicate provider rows
UI bug found by multiacct1. Both checkers must pass. Docs-only PR, merge when green
(`gh pr merge N --merge --match-head-commit <sha>`). Report the sprint list.
