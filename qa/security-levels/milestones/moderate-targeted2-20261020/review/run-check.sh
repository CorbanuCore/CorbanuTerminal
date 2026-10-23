#!/bin/bash
# usage: run-check.sh N  (check dir $W/check-rN)
N=$1; W=/Volumes/CorbanuDrive/Corbanu/.codex-work/moderate-targeted-20261019; R=$W/review
date -u +"start %FT%TZ" > $R/r$N-start.txt
~/.local/bin/corbanu exec --json -s read-only --skip-git-repo-check -m claude-opus-5-5-plan -c model_provider="claude-plan" -c model_reasoning_effort="high" -C $W/check-r$N -o $R/evidence-check-r$N.md - < $R/prompt-r$N.md > $R/r$N-events.jsonl 2> $R/r$N-stderr.txt
echo "exit=$?" >> $R/r$N-start.txt
date -u +"end %FT%TZ" > $R/r$N-end.txt
