You are the PF-23-S03 qualification worker. Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md
(binding rules). PF-23-S03 code is merged (#246-#249) but the sprint stays in_progress because its GLM 5.2 tmux
functional run and its short demo videos are missing. Following the repo's tmux SOP (docs under qa/ and the
corbanu-terminal-development skill), rebuild the binary, run the PF-23-S03 functional checks in tmux driving GLM 5.2
(provider zai), record short videos of each core feature end to end, index them under qa/demos/, then update the sprint
evidence, open a PR, merge when green (`gh pr merge N --merge --match-head-commit <sha>`), and archive PF-23-S03 if its
gate is complete. Do not touch files PR #253 (PF-24-S02) changes. Report as in sec-common.md.
