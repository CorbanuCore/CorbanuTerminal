#!/bin/bash
Q=/Volumes/CorbanuDrive/Corbanu/.codex-work/tart-par1-20261020
tail -4 $Q/runs/seq.log; memory_pressure -Q | tail -1
for r in $Q/runs/agms-*; do [ -f $r/collected.tar ] || continue; echo "$(basename $r): $({ tar -xOf $r/collected.tar evidence/result.md 2>/dev/null || tar -xOf $r/collected.tar work/evidence/result.md 2>/dev/null; } | grep -E '^[-* `]*VERDICT MOD' | sort -u | tr -d '`*-' | tr '\n' ' ')"; done
