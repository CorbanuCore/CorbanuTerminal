**Verdict: approve with fixes.** All four round-2 fixes hold. None of the four findings below lets a weaker state than the one the person reviewed be saved. Finding 1 should land before merge; findings 2–4 can be follow-ups.

This was a read-only review of `git diff origin/main...HEAD -- codex-rs`, mainly commit 5169ccfbe8. I didn't build or run anything.

**Checked and fine**
- **Downgrade check:** the new `kind` and `floor` logic in `transition.rs` (lines 203–248 and 318) works. A stored level higher than what was shown still fails with `StoredLevelChanged`. `Unchanged` and `Restrictive` save the higher of the stored level and the target, and every save merges in all revocations, kill switch included. `prepare_transition` behaves as before (floor = from).
- **Downgrade when the chosen level equals the current one (Permissive) but the stored level is Aggressive:**
  - **Level and revocation:** the session's level stays the same. The revocation (`AllActiveAuthority`) is applied, the epoch moves on, and this session's sinks are told (`closes_channels`). Other sessions aren't touched, as before.
  - **`config.toml`:** it is still rewritten when it sets a stricter level.
  - **The gap:** the level saved for the next start isn't updated in memory (finding 1).
- **Epoch binding:** `LevelBasis` equality includes `live` and `epoch`, so `reviewed.epoch` is always the current epoch when the commit starts. The `Some(_) if !live` branch can't happen. All three `AuthorityMismatch` sources now map to `Changed`, which is correct.
- **Propagation:** `rose` is computed under the tree's write lock, and `notify` runs after the lock is released. There's no new lock-ordering problem.
- **Restart arguments:** both entry points (`tui/src/main.rs` and the multitool's `run_interactive_tui`) go through `run_main`. `resume` and `fork` prompts are folded into `interactive.prompt` before that. Images live in `shared`, which `cli.images` reaches through `DerefMut`. There's no stdin prompt path. The other place that builds a prompt (`cli/src/main.rs` around line 2298) isn't a TUI and can't trigger a restart.

**Findings**

1. **Low/Medium — the level saved for the next start goes stale after a downgrade to the same level.** `core/src/security/transition.rs:417`
   - **Cause:** the update only runs when `to != from` or the kind is `Unchanged`. A `Downgrade` can now have `to == from`, so the in-memory `next_start_level` keeps its old value.
   - **How it's reached:**
     - An `Unchanged` commit merges a stricter stored level that another process saved in a race. `next_start_level` becomes that stricter level while the session stays Permissive.
     - The next Permissive choice is then a `Downgrade` with `to == from`. Core saves Permissive (and may rewrite `config.toml`), and this session's grants are revoked.
     - But `report.next_start` still shows the stale level. `confirm.rs:303-318` then restores the level files and says "Not saved: Core keeps … Nothing changed", which is false.
   - **Effect:** the level files and `security_state.json` now disagree. A second attempt heals it.
   - **Fix:** use `if prepared.to != prepared.from || matches!(prepared.kind, TransitionKind::Unchanged | TransitionKind::Downgrade)`. Add a test: a session at Permissive with `next_start_level` Aggressive and stored Aggressive, choose Permissive, expect `report.next_start == Permissive`.
   - **Also:** when Core did commit, the mismatch branch in `confirm.rs` shouldn't say "Nothing changed". It should say what Core saved.

2. **Low — the Permissive review promises a revocation that doesn't happen.** `tui/src/bottom_pane/security_level_picker.rs:1048-1058`
   - **Cause:** the branch for a live session at Permissive also catches cases that commit as `Unchanged`, which carries no revocation event:
     - stored record absent or unreadable;
     - stored Permissive while the next start isn't Permissive.
   - **Effect:** the screen says grants and "for session" approvals end, but they survive. It also says "saved by another session", and shows an absent record as "unreadable". The person is told the session is stricter than it is.
   - **Fix:** show the downgrade wording only for `StoredSecurityState::Level(l) if l > Permissive`. Otherwise say the saved record is set or repaired to Permissive and make no promise about grants. Add a snapshot test for the absent and unreadable cases.

3. **Low — the restart marker is inherited by every child process.** `tui/src/security/restart.rs:21`, `tui/src/lib.rs:1001`
   - **Effect:** shell tools, MCP servers and any nested Corbanu TUI get `CORBANU_RESTARTED_FOR_SECURITY=1`. A TUI an agent starts silently drops its prompt and images. This is the safe direction, and the disposition already notes it.
   - **Fix:** read the marker once at the top of `main()`, before any threads, then remove it (`unsafe { std::env::remove_var }` while still single-threaded). Or add it to the list of variables the shell environment drops.

4. **Info — the downgrade floor uses `max(from, reviewed_stored)` instead of `reviewed_stored`.** `transition.rs:248`
   - **Effect:** when Core's level in this session is above the stored level the person saw (for example, raised by a repository's config), a stricter level saved between `LevelBasis::read` and the store lock, up to `from`, would be lowered. The person never saw it.
   - **Why it's not reachable today:** only `/security` writes levels, and it holds `security_confirm.lock` across processes.
   - **Fix:** in `prepare_reviewed_transition`, set `floor = reviewed_stored`. An unreadable or absent record already maps to Permissive.

Not a finding: once Moderate exists, choosing it below a stricter stored level becomes `Restrictive`, saves the stricter level and is then reported as "Nothing changed". Moderate isn't available yet; recheck this when it ships.