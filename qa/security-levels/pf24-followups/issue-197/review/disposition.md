# Review disposition (Opus 5.5 High, approve with fixes)

| # | Severity | Disposition |
| - | -------- | ----------- |
| 1 | High (pre-existing) | Fixed for the session itself: `level::save(Permissive)` no longer deletes the rule file, and the next launch removes it. **Still open:** a second Permissive instance launched while an Aggressive one runs removes the file at its own launch. That instance's new threads would then lack the rule. Recorded for PF-24-S02. |
| 2 | Medium | Fixed. `thread/start` skips the fallback warning when `strict_rules` is on; the app-server test checks that no warning is sent. |
| 3 | Low | Fixed. The strict error is one line with the file and line number, and says to fix or remove the file and start a new thread. The TUI's "Failed to start a fresh session" prefix comes from the existing lifecycle code and is unchanged. |
| 4 | Low | Fixed. `child_uses_parent_exec_policy` also requires `parent.strict_rules \|\| !child.strict_rules`, with a test. |
| 5 | Low | Fixed. The TUI test is renamed `strict_rules_is_forced_and_verified`, a role test was added, and the app-server test now delivers the setting with `-c` over a user `strict_rules = false`. |
| 6 | Nit | The schema line for `ModelProviderInfo` is regeneration catching up with a doc comment already on main. Noted in the PR. |
