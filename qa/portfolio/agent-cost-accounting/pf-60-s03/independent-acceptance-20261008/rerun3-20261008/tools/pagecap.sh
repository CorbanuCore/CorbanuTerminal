#!/bin/bash
# usage: pagecap.sh <session> <outfile> : capture the currently open page (after select.sh) with full scroll
T="${TMUXCMD:-tmux -L pf60rerun3}"; C=${CAPDIR:-<scratch>/cap}; here=$(dirname "$0")
{ $here/scrollcap.sh $1 ${STEPS:-60}; } > $C/$2.txt
