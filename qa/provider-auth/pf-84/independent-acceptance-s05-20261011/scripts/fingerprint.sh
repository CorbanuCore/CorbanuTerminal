#!/bin/bash
# fingerprint.sh : the same fake key under the same provider/name in two homes, then a
# replacement key under the same name. Each run is a 401 (no spend) that still records the
# attempt's account {name, fingerprint}.
cd "$(dirname "$0")" && . ./env.sh
OUT=$EV/captures/fingerprint.txt; : > "$OUT"
K1=qa-fake-zai-4444444444444444.S05FP1 K2=qa-fake-zai-5555555555555555.S05FP2
fp() { sqlite3 "$1"/pfterminal_state_*.sqlite "select json_extract(payload,'$.account') from draft_accounting_attempts where payload like '%\"account\"%' order by json_extract(payload,'$.dispatched_at_ms')" | tr '\n' ' '; }
x() { key provider/zai_api_key env CORBANU_HOME="$1" S="$S" python3 ./cleanexec.py ZAI_API_KEY -- "$CAND" exec --skip-git-repo-check -C "$S/ws" --account same "pong" >/dev/null 2>&1; echo "exit=$?"; }
for n in fa fb; do
  H=$S/homes/$n; mkdir -p "$H"; printf 'model = "glm-5.3-flash"\nmodel_provider = "zai"\n\n[features]\nnamed_accounts = true\n' > "$H/config.toml"
  SECRET=$K1 run_secret_stdin "$CAND" "$H" account add zai same >/dev/null 2>&1
  echo "$n: same key K1 as zai/same -> exec $(x "$H"); ledger accounts: $(fp "$H")" >> "$OUT"
done
H=$S/homes/fa
SECRET=$K2 run_secret_stdin "$CAND" "$H" account add zai same 2>&1 | grep -v WARNING >> "$OUT"
echo "fa: replaced with K2 -> exec $(x "$H"); ledger accounts (in order): $(fp "$H")" >> "$OUT"
SECRET=$K1 run_secret_stdin "$CAND" "$H" account add zai same >/dev/null 2>&1
echo "fa: back to K1 -> exec $(x "$H"); ledger accounts (in order): $(fp "$H")" >> "$OUT"
grep -rl -e "$K1" -e "$K2" "$S/homes/fa" "$S/homes/fb" | sed "s#$S#\$S#" | sed 's/^/canary value found in: /' >> "$OUT" || true
echo "canary scan of fa/fb (all files): $(grep -rl -e "$K1" -e "$K2" "$S/homes/fa" "$S/homes/fb" | wc -l | tr -d ' ') files contain a canary value" >> "$OUT"
cat "$OUT"
