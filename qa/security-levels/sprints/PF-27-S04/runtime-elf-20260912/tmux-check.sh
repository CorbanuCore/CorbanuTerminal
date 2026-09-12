#!/usr/bin/env bash
set -euo pipefail
evidence=${PF27_EVIDENCE_DIR:?fresh evidence directory required}
mkdir -p "$evidence"
socket="$evidence/tmux.sock"
repo=/home/travis/worktrees/security-broker-elf-20260912
command="PF27_EVIDENCE_DIR='$evidence' bash '$repo/qa/security-levels/sprints/PF-27-S04/runtime-elf-20260912/qualify-rtx.sh'; printf '\\nparser-exit=%s\\n' \"\$?\""
printf '%s\nEnter (separate action)\n' "$command" > "$evidence/tmux-keys.txt"
tmux -S "$socket" new-session -d -s parser -x 180 -y 50 -c "$repo" 'bash --noprofile --norc'
tmux -S "$socket" send-keys -t parser:0.0 -l "$command"
tmux -S "$socket" send-keys -t parser:0.0 Enter
printf '%s\n' "$socket"
# Coordinator captures the pane and exact exit after completion; no target invocation.
