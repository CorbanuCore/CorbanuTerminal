# Review disposition (Opus 5.5 High)

## Round 1: request changes

| # | Severity | Disposition |
| - | -------- | ----------- |
| 1 | High | Fixed. The commit goes through the policy tree that holds the picker's session thread (`live_controller(home, thread)`), not the strictest tree. Choosing the level already in force now raises the other trees of the process that are below it. A downgrade's revocation ends grants in this session only, and the review says so. Tests: `security_transition_stricter_level_applies_to_the_live_session_now` (reaches the other tree) and `..._permissive_lowers_a_stricter_record_saved_elsewhere`. |
| 2 | Medium | Fixed. When the saved Core record is stricter than this session, the session takes it first, so choosing Permissive is a downgrade that lowers the record. `confirm::run` also fails and restores the files whenever Core keeps a different level for the next start. |
| 3 | Medium | Partly fixed. The Aggressive lock is released before restarting (`Mutex<Option<File>>`). Still open: on Windows the parent process doesn't ignore Ctrl-C while it waits for the restarted child. |
| 4 | Medium | Fixed. The basis records whether the session's tree runs in this process. Without it, the review says the change applies "from the next start". |
| 5 | Low/Medium | Fixed. Only an Aggressive record or corrupt content counts as tampering. A record that can't be read (for example inside a sandbox) is treated like a missing one. The repair hint works with the flag off, because a stored non-Permissive level turns the picker on. |
| 6 | Low | Fixed. `security_confirm.lock` is held across snapshot, save, commit and restore, with a 2 s wait. |
| 7 | Low | Fixed. `LevelBasis` carries the tree's `AuthorityEpoch`, so any commit after the review makes the review stale. Test: `security_transition_review_is_bound_to_the_tree_epoch`. |
| 8 | Low | Fixed. Both paths read the kill switch from the revocation state. |
| 9 | Low | Fixed in round 2 (see below). The round-1 fix, and its claim that the TUI has no image argument, were wrong. |
| 10 | Low | Partly fixed. The Permissive review says when `config.toml` will be rewritten. Still open: a profile-v2 user config path is neither read nor edited. This errs strict: the next start keeps the stricter level, and the next-start line can be wrong in that direction. |

## Round 2: request changes

| # | Severity | Disposition |
| - | -------- | ----------- |
| 1 | Medium | Fixed. Restart arguments are no longer edited. The new process gets every argument unchanged and `CORBANU_RESTARTED_FOR_SECURITY=1`, so the TUI drops its initial prompt and images (`run_main`). Test: `security_confirm_restart_keeps_every_argument_and_marks_the_process`. Limit: agent commands inherit the marker, so a TUI an agent command starts would not send its own initial prompt. |
| 2 | Low/Medium | Fixed with the reviewer's better fix. `catch_up` is gone. `prepare_reviewed_transition` takes the stored level the person reviewed: a choice below it is a downgrade of the saved record, and only a stored level above it counts as `StoredLevelChanged`. The session isn't raised. The review says "the Core level saved for the next start becomes Permissive … this session stays Permissive". |
| 3 | Low/Medium | Fixed. `propagate` tells a tree's sinks whenever its level rose, including on `Unchanged`. When the session is already Aggressive, the review says so and makes no promise about this session's approvals. Test: `security_transition_unchanged_raise_notifies_the_other_sessions`. |
| 4 | Low | Fixed. A live session's request uses the reviewed `AuthorityEpoch`, and a moved epoch maps to "review it again". |
| 5 | Process | `origin/main` was merged into the branch and the tests were run again. |
