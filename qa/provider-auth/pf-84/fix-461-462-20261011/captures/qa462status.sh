#!/bin/bash
# usage: qa462status.sh <bin> <tag>  : Z.AI-only home onboarded by a synthetic key, then /status
. /Volumes/CorbanuDrive/Corbanu/tmp/pf84fix/lib.sh
B=$1; tag=$2; s=s462-$tag; H=$S/homes/$s
rm -r "$H" 2>/dev/null < /dev/null; mkdir -p $H
tui_start $s $B $H
wait_for $s 'Provider: OpenAI Codex Account' 30 >/dev/null
select_row $s 'Provider: Z.AI API Key$' >/dev/null; key $s Enter
wait_for $s 'never adds it to chat' 20 >/dev/null
send_text $s "synthetic-zai-462-status"; settle $s; key $s Enter; wait_for $s 'API key configured' 20 >/dev/null; key $s Enter
wait_for $s 'Choose a provider account' 20 >/dev/null
select_row $s 'Done' >/dev/null; key $s Enter
wait_for $s 'Do you trust' 20 >/dev/null && key $s Enter
wait_for $s '/model to change' 40 >/dev/null || capv $s | grep -v '^\s*$' | tail -20
send_text $s "/status"; settle $s; key $s Enter; sleep 2; settle $s
echo "== $tag: /status in a Z.AI-only home (onboarded with a synthetic Z.AI key; auth.json present: $(test -e $H/auth.json && echo yes || echo no))"
capv $s | grep -E 'Model provider|Account:'
$T send-keys -t $s -l "/exit"; sleep 0.5; $T send-keys -t $s Enter; sleep 2; $T kill-session -t $s
