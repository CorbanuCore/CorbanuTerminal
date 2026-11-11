#!/bin/bash
# usage: run-agent.sh WORKDIR PROMPT OUTPREFIX   (fresh read-only Opus 5.5 High, cwd = WORKDIR)
cd "$1" || exit 1
date -u +%FT%TZ > "$3-start.txt"
~/.local/bin/corbanu exec --json --skip-git-repo-check -s read-only -m claude-opus-5-5-plan \
  -c model_provider="claude-plan" -c model_reasoning_effort="high" -C "$1" -o "$3-output.md" - < "$2" > "$3-events.jsonl" 2> "$3.stderr"
echo "exit=$?" > "$3-exit.txt"; date -u +%FT%TZ > "$3-end.txt"
