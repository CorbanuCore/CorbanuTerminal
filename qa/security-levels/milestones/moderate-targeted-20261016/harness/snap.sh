#!/bin/bash
# usage: snap.sh LABEL  -> captures/LABEL.ansi (+ prints plain)
. /Volumes/CorbanuDrive/Corbanu/.codex-work/moderate-targeted-20261016/h/cap.env
/Volumes/CorbanuDrive/Corbanu/.codex-work/moderate-targeted-20261016/h/cap.sh $CAP ansi "$1" > /Volumes/CorbanuDrive/Corbanu/.codex-work/moderate-targeted-20261016/captures/$1.ansi
sed 's/\x1b\[[0-9;]*m//g' /Volumes/CorbanuDrive/Corbanu/.codex-work/moderate-targeted-20261016/captures/$1.ansi
