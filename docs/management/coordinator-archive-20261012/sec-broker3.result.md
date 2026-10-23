# Broker lane round 3: result (2026-10-06)

## Shipped
- PR #208, merged (merge commit `24a57e38c4`, head `fb69deb9bf`). It contains:
  - PF-28-S01, the central secret and protected-output gate, behind `secret_output_gate` (default off);
  - the PF-27-S02 close-out: Linux Landlock end-to-end run, CI box ticked, decision recommendations.

## Gate evidence (qa/security-levels/sprints/PF-28-S01/README.md)
- **Focused tests:** 38 `pf_28_s01` tests pass (secret-broker 20, core 16, vault 1, login 1).
- **Affected crate suites:** 798 passed.
- **Core subset:** 1439/1442. The 3 failures also fail on a clean `origin/main`.
- **TUI subset:** 104/105. The `claude_auth` tmux test times out because the long worktree path truncates the status line.
- **GLM 5.2 TUI runs (tmux, demo SOP):** four published videos at `9261bd416f`, indexed in `qa/demos/index/PF-28-S01.md`:
  - tool-output-gated
  - split-output
  - persistence-clean
  - baseline-flag-off
- **Review:** Opus 5.5 High, six rounds. Rounds 1–5 asked for changes, and all of them were fixed or recorded. Round 6 gave APPROVE WITH NITS, and its nits are recorded. Review files: `qa/security-levels/sprints/PF-28-S01/review-opus-{1..6}.md`.
- **CI:** all checks green at merge. Flaky Windows/macOS canary and Windows contract jobs were re-run. A real argument-comment-lint failure was fixed.

## Open
- **Sprint record stays `draft`.** The lifecycle check needs PF-27-S02 archived before PF-28-S01 can be `in_progress` or `completed`. PF-27-S02 is waiting on Travis's two decisions; my recommendations are recorded in its sprint file.
- **Remaining (sprint file), cut down:**
  - decode-and-rescan (wrapped encodings) and reflected values: PF-28-S02;
  - MCP OAuth refresh registration;
  - comma-separated or numbered seed phrases;
  - per-field scrubbing instead of a serde round trip;
  - per-session stream state and performance.
- **Unrelated pre-existing failures:**
  - `skills_*` developer-message tests (2);
  - `remote_compact_trim_estimate_uses_session_base_instructions`;
  - the claude_auth tmux test with long paths.
