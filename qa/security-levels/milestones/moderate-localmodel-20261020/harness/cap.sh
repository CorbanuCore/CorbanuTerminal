#!/bin/bash
# usage: cap.sh RUN_ID agms-tui-args...   (coordinator capture lane agmx0; packet screens and shakedown only, never a case)
Q=/Volumes/CorbanuDrive/Corbanu/.codex-work/qwen-1052-20261020
export TART_QA_WORK=/Volumes/CorbanuDrive/Corbanu/.codex-work/tart-eval
. /Volumes/CorbanuDrive/Corbanu/worktrees/qwen-1052/scripts/qa/tart/lib.sh
R="$1"; shift
q=""; for a in "$@"; do q="$q '${a//\'/\'\\\'\'}'"; done
exec ssh -n "${SSH_OPTS[@]}" "agent@$(cat $Q/common/vm-ip.txt)" "cd /tmp; sudo -n -u agmx0 -H /opt/agms/bin/agms-launch tui $R$q"
