RECOVERY NOTE (coordinator, 2026-10-11): a previous run of this brief was STOPPED by the coordinator because Travis decided
#442 differently from what it was building. Its work is in the worktree
/Volumes/CorbanuDrive/Corbanu/worktrees/aggressive-p1-sec-20261011 (a "wip all" commit plus untracked demo specs/wrappers),
build env /Volumes/CorbanuDrive/Corbanu/.codex-work/targets/aggp1*; nothing was pushed as a PR. Inspect it and keep what's
sound for #434, #440 and #441.
#442 DECISION (Travis, comment on #442): under Aggressive, reads of protected credential files (e.g. .env) are NEVER
grantable, not even once. Do NOT make protected reads offer "Grant once"; remove any such change and the
issue442-protected-read-offers-grant demo. Instead: explain it plainly at the point of denial (and in the inspector /
review text), naming the security level and how the user can change it; keep "Grant once" for other gated operations
(e.g. protected writes). Close #442 with that behaviour and evidence. Note: the separate worker agg-fix-b owns #439
(refusal attribution wording); coordinate by keeping your #442 explanation scoped to protected reads and merging
origin/main often.

You are a P1 security bug-fix worker for the "Aggressive ships" milestone. Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md (binding; "Linux box" section: Linux work on glitch, `ssh glitch`), the milestone record qa/security-levels/milestones/aggressive-20261010/README.md (on main once PR #447 merges, else on branch qa/aggressive-milestone-20261010) and, for each of your issues, its body and the cited evidence under that record (runs/<group>/). Do NOT read or edit the frozen design or results of the milestone; you are fixing the product, not the test.
Fix, highest severity first: #434 (vault-managed secret shown verbatim in command output despite the output gate — verify
whether a mid-session vault add is covered, and after restart; the gate must cover both or the UI must say when it applies),
#442 ("Grant once" unreachable for protected reads such as cat .env), #440 (ordinary environment variables removed too; only
secret-like ones should be), #441 (nested corbanu exec/review should be refused before the approval prompt, not after).
Do NOT change behaviour for #443 or #445 — those await Travis's product decisions; another worker handles the UX issues
#435-#439, #444.
Per fix: regression tests that reproduce the milestone observation (fail before, pass after); focused `just test` with and without developer-accounting; Linux clippy -D warnings on glitch; one independent Opus 5.5 High read-only review via installed `corbanu exec` per PR; a tmux functional check (docs/tmuxHarness.md) with GLM 5.3 Flash (`-m glm-5.3-flash -c model_provider="zai"`, key via `ZAI_API_KEY="$(env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME ~/.local/bin/corbanu vault auth-helper provider/zai_api_key)"` on the consuming command) in a disposable home (CORBANU_TEST_NO_NATIVE_KEYRING=1) reproducing the milestone steps before/after; short SOP videos for user-visible changes (qa/demos/README.md, leak-scanned). If the Anthropic content filter stops you, say so in your final report. New worktrees off origin/main under /Volumes/CorbanuDrive/Corbanu/worktrees/; one PR per issue or small related group; merge only when ALL checks are green, never with checks pending (`gh pr merge N --merge --match-head-commit <sha>`; merge origin/main on conflicts; never force-push; never rm -f). The flaky Windows probe test #446 may fail unrelated PRs: rerun the failed job once (`gh run rerun <id> --failed`) before investigating. Close each issue with evidence. Remove build targets when done. Report as in sec-common.md, listing anything Travis must decide.
