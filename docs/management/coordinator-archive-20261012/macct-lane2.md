You are the lane owner for PF-84 (named accounts per provider), continuing after the previous lane worker. Read
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md (binding), macct-lane.md (the original brief —
its rules, gate and requirements all still apply) and macct-lane.result.md (what shipped: plan activated #407; S01 #408 and
S02 #409 merged and `ready`; S02 behind the default-off `named_accounts` flag; open items and known limits) in that folder,
the corbanu-terminal-development skill, and on origin/main docs/plans/active/unified-provider-auth.md (PF-84 section),
qa/provider-auth/pf-84/s01-gate.md, s02-gate.md and the S03/S04/S05 sprint records.
Do next, in order: S03 (account per session, `corbanu exec --account`, an `account` option on spawned agents incl. tmux
workers, resume uses the recorded account, refuse missing/failed accounts — never fall back; also fix the known limit
that the TUI worker pre-check tests the default account's command), then S04 (`/providers` account management +
onboarding + the duplicate provider rows bug; decide with evidence whether named ChatGPT/OpenAI sign-ins and AWS profile
accounts fit S04 or should be split out — write the trade-off for Travis rather than silently dropping them), then S05
(per-account usage and rate-limit attribution; PF-60-S04 is archived so its dependency is met). First a small docs PR to
allocate S03's worktree/branch/base. Per-slice and per-sprint gates exactly as in macct-lane.md (GLM 5.3 Flash runs with a
real default Z.AI key + fake named account 401; a second provider; tmux per docs/tmuxHarness.md; SOP videos; one Opus 5.5
High review; merge only when ALL checks are green, never with checks pending; never force-push; never rm -f).
NOTE: the RTX box (ssh ambient@100.99.88.49) is OFFLINE (Tailscale shows it offline). Try it once per sprint; if still
unreachable, rely on the PR's Ubuntu clippy job (-D warnings) and say so in the gate file. Also, when it's back, remove the
previous worker's leftovers there: ~/corbanu-rtx/pf84-wt (git worktree remove), pf84-target, pf84-clippy.sh, pf84-* logs.
Set each sprint to `ready` when its gate passes; do NOT archive. Stop and report after S04 is ready if context is getting
large. Keep context lean. Remove build targets when done. Report as in sec-common.md, listing anything Travis must decide.
