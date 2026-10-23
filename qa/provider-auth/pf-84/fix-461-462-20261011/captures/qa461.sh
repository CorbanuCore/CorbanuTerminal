#!/bin/bash
# usage: qa461.sh <bin> <tag> <field: name|key|default>
. /Volumes/CorbanuDrive/Corbanu/tmp/pf84fix/lib.sh
B=$1; tag=$2; field=$3; s=q461-$tag-$field; H=$S/homes/$s
rm -r "$H" 2>/dev/null < /dev/null; mkdir -p $H
printf '[features]\nnamed_accounts = true\n' > $H/config.toml
tui_start $s $B $H
wait_for $s 'Provider: OpenAI Codex Account' 30 >/dev/null
select_row $s 'Provider: Z.AI API Key$' >/dev/null; key $s Enter
wait_for $s 'never adds it to chat' 20 >/dev/null
if [ "$field" = default ]; then
  send_text $s "qsynthetic-461-default-QQ"; sleep 2; settle $s
  echo "== $tag default-key field after typing 'qsynthetic-461-default-QQ'"; capv $s | grep -v '^\s*$' | tail -12
  $T kill-session -t $s; exit 0
fi
send_text $s "synthetic-zai-default-461"; settle $s; key $s Enter; wait_for $s 'API key configured' 20 >/dev/null; key $s Enter
wait_for $s 'Choose a provider account' 20 >/dev/null
select_row $s 'Provider: Z.AI( |$)' >/dev/null; key $s Enter
select_row $s 'Add another account' >/dev/null; key $s Enter
wait_for $s 'Name it' 10 >/dev/null
if [ "$field" = name ]; then
  send_text $s "qa-probe"; sleep 2; settle $s
  echo "== $tag account-name field after typing 'qa-probe'"; capv $s | grep -v '^\s*$' | tail -10
else
  send_text $s "probe"; settle $s; key $s Enter; wait_for $s 'never shown' 10 >/dev/null
  send_text $s "abc-q-synthetic-461-secret-tail"; sleep 2; settle $s
  echo "== $tag account-key field after typing 'abc-q-synthetic-461-secret-tail'"; capv $s | grep -v '^\s*$' | tail -10
fi
$T kill-session -t $s
