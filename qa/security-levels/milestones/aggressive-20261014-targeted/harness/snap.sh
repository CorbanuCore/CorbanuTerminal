#!/bin/bash
# usage: snap.sh LABEL  -> captures/LABEL.ansi (+ prints plain)
. /Volumes/CorbanuDrive/Corbanu/.codex-work/aggressive-targeted-20261014/h/cap.env
/Volumes/CorbanuDrive/Corbanu/.codex-work/aggressive-targeted-20261014/h/cap.sh $CAP ansi "$1" > /Volumes/CorbanuDrive/Corbanu/.codex-work/aggressive-targeted-20261014/captures/$1.ansi
sed 's/\x1b\[[0-9;]*m//g' /Volumes/CorbanuDrive/Corbanu/.codex-work/aggressive-targeted-20261014/captures/$1.ansi
