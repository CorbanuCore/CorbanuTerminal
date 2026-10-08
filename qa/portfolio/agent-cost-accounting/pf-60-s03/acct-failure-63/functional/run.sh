#!/bin/bash
# usage: run.sh N
cd /Volumes/CorbanuDrive/Corbanu/.codex-work/acct63/func
~/.local/bin/corbanu exec -s read-only -c approval_policy=never --skip-git-repo-check -m claude-opus-5-5-plan -c model_provider=claude-plan -c model_reasoning_effort=low -C /Volumes/CorbanuDrive/Corbanu/.codex-work/acct63/func - < prompt.md > exec-$1.log 2>&1
echo EXIT:$? >> exec-$1.log
