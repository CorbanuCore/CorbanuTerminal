#!/bin/bash
# flagoff.sh : named_accounts off (the default). Identical fresh homes for the candidate (hf)
# and the pre-S05 baseline (hf0). Account `work` is enrolled with the flag on, then every run
# has the flag off: default, and `--account work` (ignored with the flag off, per S02).
cd "$(dirname "$0")" && . ./env.sh
OUT=$EV/captures/flag-off.txt; : > "$OUT"
for pair in "hf:$CAND:CAND" "hf0:$BASE:BASE"; do
  IFS=: read -r name b who <<<"$pair"; H=$S/homes/$name; mkdir -p "$H"
  printf 'model = "glm-5.3-flash"\nmodel_provider = "zai"\n' > "$H/config.toml"
  echo "== $name ($who) ==" >> "$OUT"
  key provider/zai_api_key run_secret_stdin "$b" "$H" account add zai work --enable named_accounts 2>&1 | grep -v WARNING >> "$OUT"
  echo "  account add with the flag off: $(SECRET=x run_secret_stdin "$b" "$H" account add zai other 2>&1 | grep -v WARNING | head -1)" >> "$OUT"
  for args in "" "--account work"; do
    key provider/zai_api_key env CORBANU_HOME="$H" S="$S" python3 ./cleanexec.py ZAI_API_KEY -- "$b" exec --skip-git-repo-check -C "$S/ws" $args \
      "Reply with exactly the word pong and nothing else." > "$S/logs/fo-$name.out" 2> "$S/logs/fo-$name.err"
    echo "  exec $args -> exit=$? reply=$(tr -d '\n' < "$S/logs/fo-$name.out" | cut -c1-20); warnings: $(grep -iE 'account|named' "$S/logs/fo-$name.err" | grep -v call_metrics | sort -u | cut -c1-200 | tr '\n' ' ')" >> "$OUT"
  done
  db=$(ls "$H"/pfterminal_state_*.sqlite)
  echo "  ledger format tables: $(sqlite3 "$db" "select group_concat(name) from sqlite_master where name like 'draft_accounting_ledger_format%'"); attempts $(sqlite3 "$db" 'select count(*) from draft_accounting_attempts'); with account field $(sqlite3 "$db" "select count(*) from draft_accounting_attempts where payload like '%\"account\"%'")" >> "$OUT"
  echo "  throttle keys: $(sqlite3 "$db" "select group_concat(key_fingerprint, ', ') from provider_request_state" | sed -E 's/(env:[A-Z_]+:)[0-9a-f]+/\1<hash>/g')" >> "$OUT"
done
sed "s#$S#\$S#g" "$OUT"
