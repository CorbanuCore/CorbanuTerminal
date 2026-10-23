#!/bin/bash
# usage: snap.sh LABEL  -> captures/LABEL.ansi (+ prints plain)
. /Volumes/CorbanuDrive/Corbanu/.codex-work/aggressive-rerun-20261011/h/cap.env
/Volumes/CorbanuDrive/Corbanu/.codex-work/aggressive-rerun-20261011/h/cap.sh $CAP ansi "$1" > /Volumes/CorbanuDrive/Corbanu/.codex-work/aggressive-rerun-20261011/captures/$1.ansi
sed 's/\x1b\[[0-9;]*m//g' /Volumes/CorbanuDrive/Corbanu/.codex-work/aggressive-rerun-20261011/captures/$1.ansi
