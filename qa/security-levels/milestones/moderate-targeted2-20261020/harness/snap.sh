#!/bin/bash
# usage: snap.sh LABEL  -> captures/LABEL.ansi (+ prints plain)
. /Volumes/CorbanuDrive/Corbanu/.codex-work/moderate-targeted-20261019/h/cap.env
/Volumes/CorbanuDrive/Corbanu/.codex-work/moderate-targeted-20261019/h/cap.sh $CAP ansi "$1" > /Volumes/CorbanuDrive/Corbanu/.codex-work/moderate-targeted-20261019/captures/$1.ansi
sed 's/\x1b\[[0-9;]*m//g' /Volumes/CorbanuDrive/Corbanu/.codex-work/moderate-targeted-20261019/captures/$1.ansi
