#!/bin/bash
W=/Volumes/CorbanuDrive/Corbanu/.codex-work/aggressive-targeted3-20261015
tail -1 $W/runs/seq.log
for R in $W/runs/agms-G*b-* $W/runs/agms-G17c-* $W/runs/agms-G21*; do [ -f $R/executor-exit.txt ] || continue; echo "$(basename $R): $( (tar -xOf $R/collected.tar evidence/result.md 2>/dev/null; tar -xOf $R/collected.tar work/evidence/result.md 2>/dev/null) | grep -o -E 'VERDICT AGG-[A0-9]+: (passed|failed|blocked)' | sort -u | tr '\n' ' ')"; done 2>/dev/null
exit 0
