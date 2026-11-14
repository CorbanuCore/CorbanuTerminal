# B4 post-grading verification (bench-1114; graders had already finished)

Both graders said solution X (Opus) makes `! { echo ok; } | bash` and `time { echo ok; } | bash` quieter than before. Verified with a scratch probe (`trackB/hidden/b4/probe*.rs`, appended temporarily to tainted_action_brace_reading_tests.rs, then reverted), `classify_with` / `classify_as_named_with`:

| Line (bash and zsh, plain and named) | Base 6cf404cf18 | Opus 3c7187492c | Astra 9492f05d78 |
|---|---|---|---|
| `! { echo ok; } \| bash` | UnseenCode | **None** (quieter) | UnseenCode (its test asserts it) |
| `time { echo ok; } \| bash` | UnseenCode | **None** (quieter) | UnseenCode (its test asserts it) |
| `! echo ok \| bash` (control) | UnseenCode | UnseenCode | — |
| `echo ok \| bash` (control) | None | None | — |
| `{ echo ok; } \| bash` (control) | None | None | — |

Conclusion: the graders' finding is correct. Opus's B4 change is quieter than the base on 4 shell readings, which breaks the task's "never quieter" rule. It also makes the grouped `!` form disagree with the ungrouped `! echo ok | bash`. The payloads here are harmless, but the rule exists for exactly this reason.
