You are the PF-84 integration worker. Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md
(binding; "Linux box" section: Linux work on glitch, `ssh glitch`), macct-lane.result.md, macct-lane2.result.md,
macct-acc12.result.md and macct-acc3.result.md in that folder, the corbanu-terminal-development skill (archive procedure),
and on origin/main the PF-84 plan section, the S01/S02/S03 records, both independent acceptance READMEs under
qa/provider-auth/pf-84/, and issue #428 with Travis's decision comment.
Travis decided on 2026-10-11 (chat with the coordinator):
(1) ACCEPT PF-84-S01, S02 and S03 — S02 and S03 behind the default-off `named_accounts` flag, S03 with limits: #425 and
    #426 (missing-account refusals; being fixed by worker macct-fix2 in its own PR) and #428 must be resolved before the
    flag is removed; S02-15 stays open.
(2) #428 = Option 1: D3 (approval for a model-chosen spawn `account` switch) also applies whenever the TUI shows
    Aggressive, including "Aggressive enforced, boundary unverified" / "Active in this session: Aggressive" — not only
    when core's level is Aggressive. With approvals off, refuse.
Do, as two PRs:
A. Implement #428 Option 1 with regression tests (each of the states from #428's evidence; approvals-off refusal). Gate:
   focused tests with and without developer-accounting; Linux clippy -D warnings on glitch; one independent Opus 5.5 High
   read-only review via installed `corbanu exec`; a tmux check with GLM 5.3 Flash (`-m glm-5.3-flash -c
   model_provider="zai"`, `provider/zai_api_key`) in a disposable home reproducing the #428 state before/after; a short SOP
   video. Coordinate with macct-fix2 (branch in worktrees/pf84-account-refusals-20261010) and macct-lane3 (S04, worktrees/
   pf84-s04-providers): stay out of their scope, merge origin/main before pushing. Close #428 with evidence.
B. Docs PR: archive S01, S02 and S03 per procedure (move to docs/sprints/archive/unified-provider-auth/, status completed,
   "Closure — 2026-10-11" with Travis's acceptance and the limits above, links to gate files, acceptance READMEs and PRs
   #407 #408 #409 #411 #420 #421 #422 #429 and the fix PRs), update the plan/index/navigation; keep #425/#426/#428 (until
   fixed) and S02-15 as follow-ups. Run `python3 docs/plans/check.py && python3 docs/sprints/check.py`; must pass. Touch
   only S01-S03 records and their plan/index lines (macct-lane3 edits S04/S05).
Merge only when ALL checks are green, never with checks pending (`gh pr merge N --merge --match-head-commit <sha>`; never
force-push; never rm -f). Keys only via `X="$(env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME ~/.local/bin/corbanu vault auth-helper <label>)" <cmd>`;
never print keys. Remove build targets when done. Report as in sec-common.md.
