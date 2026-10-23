# agg-dec-env result (2026-10-11/12)

Shipped: #520 merged (6ebfa1f47c). Decisions recorded on #434, #440 and #442; summary comment on #442.
Record: qa/security-levels/fixes/issue-442-decisions-20261011/README.md. Evidence: .codex-work/targets/agg-dec-env-evidence
(tmux runs, review rounds r1-r5, test logs); glitch logs in ~/corbanu-rtx/aggdec-logs-archive.

- `.env`, `.env.*`, `.envrc` created after start are protected: Seatbelt regex (no read/create/link/write, any case, anywhere),
  Linux mask + refuse unsafe entries + inotify stop-on-create (fail closed, race stated), Windows documented.
- `.codex`/`.agents` never lifted by Grant once (Core `never_lifted`); texts updated.
- Gate: regression tests fail before; `just test` with/without developer-accounting; clippy -D warnings macOS and glitch;
  tmux before/after on GLM 5.3 Flash; Opus 5.5 High review r4/r5 APPROVE; two published videos.

For Travis: `.env.*` breadth (.env.example etc.), Linux race, Windows mid-command files, Grant once lifts almost nothing
under the real profile, credential-named links refused.
