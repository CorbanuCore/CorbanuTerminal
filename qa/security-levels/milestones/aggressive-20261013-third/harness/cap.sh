#!/bin/bash
# usage: cap.sh RUN_ID agms-tui-args...   (coordinator capture lane agmx0)
R="$1"; shift
q=""; for a in "$@"; do q="$q '${a//\'/\'\\\'\'}'"; done
exec /Volumes/CorbanuDrive/Corbanu/.codex-work/aggressive-third-20261013/h/vmssh.sh -n agent@192.168.64.3 "cd /tmp; sudo -n -u agmx0 -H /opt/agms/bin/agms-launch tui $R$q"
