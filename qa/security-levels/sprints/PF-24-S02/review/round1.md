**Verdict: request changes.** I found no way for a model, tool, app-server request or agent command to start or forge a transition. I also found no ordering, crash or failure that leaves a protection level weaker than the one the person reviewed. The blocking problems are that the screens can claim Core protection that isn't in force, and that a normal Permissive choice can trip the tamper check.

This was a read-only review: I read the diff against `origin/main` plus the uncommitted changes, and didn't build or run any tests.

**Checked and fine**
- **Trigger path:** `commit_human_level_change` and `RestartForSecurityLevel` are reached only from the picker's confirm and `r` keys.
- **Preflight:** `Probes::Passed` is only reached with `raise_core`, which requires a passed preflight recheck.
- **Ordering:** each save order fails safe — a crash or failure ends either stricter or with a startup warning.
- **Unix restart:** the restarted process doesn't inherit lock files, because Rust opens files close-on-exec.
- **Flag off:** with `security_levels` off, the picker and confirmation are unchanged. The only flag-off change is in finding 5.

**Findings**

1. **High — the commit uses the strictest policy tree in the process, not this session's.** `core/src/security/transition.rs:468-485` (`live_controller`), `core/src/security/level_change.rs:79,206`, `transition.rs:338`.
   - **How it fails:** another tree on the same home can be Aggressive, for example through a project's `[security] level`, a `/new` session or a detached spawned agent. If this session is Permissive and the person confirms Aggressive, the commit runs as `Unchanged`, which saves Aggressive but never propagates. This session stays Permissive.
   - **What the screens say:** the review promised "Core's level becomes Aggressive as soon as you confirm, in every session", and the saved screen shows "Core's level: Aggressive."
   - **Downgrades too:** the "end all active authority" revocation is applied only to that other tree. This session's grants survive, despite the review saying they end.
   - **Fix:** commit through the current session's tree, passed in from the chat widget, and show its level in the review. Also propagate on `Unchanged` when the saved level goes up (propagation only ever raises), and notify revocation sinks on every tree for a downgrade.

2. **Medium — a legitimate Permissive choice can trip the tamper check.** `level_change.rs:208ff`, together with `transition.rs` (the `Unchanged => stored_level.max(to)` merge).
   - **How it fails:** this process's tree is Permissive while the saved Core record is Aggressive (another process confirmed Aggressive). Choosing Permissive commits as `Unchanged`, so the record stays Aggressive while the level file becomes Permissive.
   - **What the person sees:** "Saved: Permissive / Core's level: Permissive". At the next start the mismatch reads as Invalid: Aggressive is enforced, nested launches are refused, and a "changed outside /security" warning appears.
   - **Fix:** in `commit_human_level_change`, use the higher of the tree's level and the saved level as the starting point, so this commits as a downgrade. At minimum, have `confirm::run` treat `report.next_start != core_level(target)` as a failure and restore the files.

3. **Medium — Windows "restart now" doesn't actually downgrade.** `tui/src/security/restart.rs:37`, `tui/src/security/launch.rs:32,50`.
   - **How it fails:** on Windows the old process stays alive while the new one runs, and it still holds `AGGRESSIVE_LOCK` (a `OnceLock` that is never released).
   - **Result:** the new Permissive process sees `aggressive_running`, keeps the rule file and registry entry, and nested launches stay refused. The parent may also handle the user's Ctrl-C itself while it waits for the child.
   - **Fix:** store the lock as a `Mutex<Option<File>>`, drop it before spawning or re-executing, and ignore console Ctrl-C in the parent while it waits.

4. **Medium — with no live tree in this process, the review over-promises.** `security_level_picker.rs:1010` (review lines) and `level_change.rs:206` (stored path).
   - **When:** the TUI is attached to a remote server or the implicit local daemon, or no session has started yet.
   - **How it fails:** the stored path only saves, and the review's basis comes from the files rather than the server's actual in-force level. The review still says "becomes Aggressive as soon as you confirm"; only the saved screen correctly says "from the next start".
   - **Fix:** work out whether a live tree exists when the review opens, and word the review to match. Better, turn `/security` confirmation off when the TUI isn't running the server in-process.

5. **Low/Medium — the tamper check can block users who never chose Aggressive.** `security-level/src/level.rs:193-209`.
   - **How it fails:** any read error on `security_state.json` makes a Permissive or absent level file read as Invalid, which enforces Aggressive and refuses nested launches. That includes permission denied, for example a child running inside Core's protected-path sandbox at Moderate.
   - **Flag off:** a leftover Aggressive record forces Aggressive, and the startup message says "choose a level in /security" even though that picker doesn't exist with the flag off.
   - **Fix:** treat only an "aggressive" record or corrupt content as Invalid. For read errors, warn without forcing Aggressive, or skip the check for nested launches. Give flag-off users a repair hint that works for them.

6. **Low — concurrent confirmations can clobber each other.** `tui/src/security/confirm.rs:164,211`.
   - **How it fails:** `Snapshot::restore` writes the old files back with no lock, so a concurrent confirm in another process can be undone.
   - **Result:** the level file and Core record can end up disagreeing (fails closed, but the other process's choice is lost).
   - **Fix:** hold a separate `security_level.toml` lock across snapshot, save, commit and restore. It must not be Core's `security_state.lock`, or the process would block on its own lock.

7. **Low — the review isn't bound to an epoch.** `level_change.rs:208`.
   - **How it fails:** the epoch is read at commit time, not at review time. Two confirmations in the same process could both pass the basis check, and the downgrade would silently undo the other's Aggressive save.
   - **Fix:** capture the `AuthorityEpoch` (and tree identity) in `LevelBasis` when the review is shown, and refuse with `Changed` if it doesn't match.

8. **Low — the policy basis disagrees with itself when the saved state is unreadable.** `level_change.rs:95`.
   - **How it fails:** without a live tree the basis reports `kill_switch_active` from "unreadable"; with a live tree it reports the tree's revocations, which are false (the root uses `force_deny` instead). A tree appearing or disappearing between review and commit then gives a spurious "changed".
   - **Fix:** use one rule for both paths.

9. **Low — restart arguments.** `restart.rs:13`.
   - **Prompt removal:** it matches by value, so it can remove a later option value that equals the prompt (`codex "fix" -m fix`).
   - **Kept arguments:** `--image` stays, so images may be sent again, and a stdin prompt (`-`) waits on the terminal.
   - **Fix:** rebuild the arguments from the parsed CLI, dropping the prompt and images, instead of editing argv.

10. **Low — undisclosed `config.toml` edit.** `core/src/security/recovery.rs:285`.
    - **How it fails:** a downgrade can rewrite `[security] level` in the user's `config.toml`, and the Permissive review never mentions it. A profile-v2 user config path is neither read nor edited, so the "from the next start" line can be wrong (in the stricter direction).
    - **Fix:** say so in the review when `user_config_needs_level` is true, and use the resolved user config path.

The blocking fixes are 1 and 2; 3 and 4 should land before a Windows or daemon-mode handoff. Each needs a test.