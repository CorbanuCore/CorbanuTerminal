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
| 9 | Low | Fixed. Restart arguments drop exactly the values clap parsed as the prompt, at any subcommand level. The TUI has no image argument. |
| 10 | Low | Partly fixed. The Permissive review says when `config.toml` will be rewritten. Still open: a profile-v2 user config path is neither read nor edited. This errs strict: the next start keeps the stricter level, and the next-start line can be wrong in that direction. |
