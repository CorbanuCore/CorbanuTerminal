# #584: no question about a .codex/.agents write that no grant lifts (Aggressive)

Follow-up from the review of #580 (#565). Under Aggressive the project's `.codex` and `.agents` stay read-only whatever
anyone answers (#520, Travis 2026-10-11).

| Route | Before (main 068c50a85e) | After |
| --- | --- | --- |
| `request_permissions` for a `.codex`/`.agents` write | asked the person (or the automatic reviewer), then dropped the entry | refused before anyone is asked; the person and the model are told why; other entries of the same request are still asked about, without it |
| A command that writes only `.codex`/`.agents` (`printf x > .codex/notes.toml`) | approval prompt offered Grant once, which could not apply | no Grant once; the prompt and the denial say it cannot be granted, not even once |

A command that also writes a path a grant lifts (a git hook) keeps the Grant once offer (`pf_25_s01`
`issue_442_a_confirmed_grant_never_lifts_codex_or_agents`).

## Evidence

- Regression tests (fail before, pass after): `core/tests/suite/issue_584.rs` (real Seatbelt on macOS, bubblewrap on glitch).
- tmux (GLM 5.3 Flash on zai, debug build of c2aa460ac3, disposable home and HOME, `CORBANU_TEST_NO_NATIVE_KEYRING=1`,
  Aggressive after restart): `after.txt`.
- Video: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-584-issue584-agent-settings-no-grant-c2aa460ac3e2-2026-10-13.mp4
- Limit (as #620): the write-target parsing is text-based; a miss offers Grant once as before, a false match only
  withholds it.
