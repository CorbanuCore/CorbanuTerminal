#!/bin/bash
# usage: qa462loop.sh <bin> <tag> : config names kimi-code; save a synthetic key; Enter
. /Volumes/CorbanuDrive/Corbanu/tmp/pf84fix/lib.sh
B=$1; tag=$2; s=l462-$tag; H=$S/homes/$s
rm -r "$H" 2>/dev/null < /dev/null; mkdir -p $H
printf 'model = "kimi-k3"\nmodel_provider = "kimi-code"\n' > $H/config.toml
tui_start $s $B $H
wait_for $s 'Use your Kimi Code API key' 30 >/dev/null
send_text $s "kimi-synthetic-462-abcdef"; settle $s; key $s Enter
wait_for $s 'API key configured' 20 >/dev/null; key $s Enter; sleep 3; settle $s
echo "== $tag: config names kimi-code; after saving the key and pressing Enter"
capv $s | grep -v '^\s*$' | tail -8
capv $s | grep -q 'Do you trust' && { key $s Enter; wait_for $s '/model to change' 30 >/dev/null; echo "-- after trusting the directory"; capv $s | grep -v '^\s*$' | tail -4; }
$T kill-session -t $s
