# #626 follow-up: the suspended-approval note for zsh-fork commands

Owner decision (Travis, 2026-10-13): the note "A remembered approval … was not applied because security level Aggressive is on …" also appears for commands intercepted by zsh-fork, whenever zsh-fork is on.

## tmux check
GLM 5.3 Flash on zai, disposable home. The debug build of `98ba145fa0` was laid out as a package whose `codex-resources/zsh/bin/zsh` is the zsh-fork build. `shell_zsh_fork` and `unified_exec_zsh_fork` were on.

1. **Under Permissive:** `touch remembered.txt`, approved with "don't ask again". The rule is saved in the home's `rules/default.rules`.
2. **Switch to Aggressive and restart:** the model runs `f=remembered.txt; touch "$f"`.
   - The shell command is approved once. It matches no rule, so there's no note.
   - zsh-fork then intercepts `/usr/bin/touch remembered.txt`. Its prompt carries the note naming `touch remembered.txt` ([final screen](final-screen.txt)).

Video: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-626-issue626-zsh-fork-remembered-approval-note-98ba145fa017-2026-10-13.mp4

## Tests
`core/tests/suite/issue_626.rs`, `issue_626_zsh_fork_prompt_names_the_suspended_approval` and its unified-exec variant. They use a real zsh-fork runtime and Seatbelt. They fail without the fix and pass with it.
