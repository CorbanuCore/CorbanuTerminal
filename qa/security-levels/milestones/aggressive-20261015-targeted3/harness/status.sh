#!/bin/bash
W=/Volumes/CorbanuDrive/Corbanu/.codex-work/aggressive-targeted3-20261015
tail -2 $W/runs/seq.log
for R in $W/runs/agms-G*; do
  [ -f $R/executor-exit.txt ] || continue
  echo "$(basename $R) $(cat $R/executor-exit.txt) $(cat $R/executor-start.txt)-$(cut -c12- $R/executor-end.txt) guard:$(tail -1 $R/guard-after.txt)"
  (tar -xOf $R/collected.tar evidence/result.md 2>/dev/null; tar -xOf $R/collected.tar work/evidence/result.md 2>/dev/null) | grep -o -E "VERDICT AGG-[A-Z0-9]+: (passed|failed|blocked)" | sort -u | tr '\n' ' '; echo
done
