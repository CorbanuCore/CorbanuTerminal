#!/bin/bash
# usage: res.sh GROUP -> prints the executor's result.md for that group's run
W=/Volumes/CorbanuDrive/Corbanu/.codex-work/moderate-first-20261014
for R in $(ls -d $W/runs/agms-$1-*); do f=$(tar -tf $R/collected.tar | grep "evidence/result.md$" | head -1); echo "#### $(basename $R)"; tar -xOf $R/collected.tar $f; done
