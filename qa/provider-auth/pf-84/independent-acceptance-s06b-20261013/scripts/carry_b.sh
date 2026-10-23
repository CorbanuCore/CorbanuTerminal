#!/usr/bin/env bash
# C1/C2 for slice B: an existing single-account home made by BASE (default ChatGPT login in auth.json, no [features]),
# then the candidate with the flag off, on for one run, and --account default; snapshot before/after. Flag-off parity of
# the named-account commands, BASE vs CAND. Output: captures/carry-flagoff.txt
set -u
D=$(cd "$(dirname "$0")" && pwd); source "$D/env.sh"; source "$D/common_b.sh"
P=34725; U=http://127.0.0.1:$P; LOG=$S/logs/mock-c.jsonl; : > "$LOG"
python3 "$D/mock.py" --port $P --log "$LOG" &
MOCK=$!; trap 'kill $MOCK' EXIT; sleep 1
export FH=$S/fh-hm H=$S/homes/hm; for d in $H $FH; do [ -d $d ] && rm -r $d < /dev/null; done; mkdir -p $H $FH
printf 'model = "gpt-5.4"\nmodel_provider = "openai"\nopenai_base_url = "%s/v1"\nchatgpt_base_url = "%s/backend-api/codex"\n[projects."%s/ws"]\ntrust_level = "trusted"\n' $U $U $S > $H/config.toml
XENV=(CODEX_REFRESH_TOKEN_URL_OVERRIDE=$U/oauth/token CORBANU_TEST_LOGIN_ISSUER=$U)
snap_() { (cd $H && find . -type f ! -path './log/*' ! -path './sessions/*' ! -path './tmp/*' ! -name '*.sqlite*' ! -name '*.lock' ! -name 'models_cache.json' ! -path './models-cache/*' -print0 | sort -z | xargs -0 shasum -a 256 | cut -c1-16,65-); }
{
echo "## home hm made by BASE $(cat $S/bin/COMMIT-base): default ChatGPT login (mock user legacy), no [features] table"
ctl '{"next_user":"legacy","ttl":3600,"refresh_ttl":3600}'
run $BASE $H login --device-auth --experimental_issuer $U < /dev/null 2>&1 | tail -1
BIN=$BASE ex | grep -vE '"ev": "get"'
echo "## snapshot S0 (sha256 prefixes; excludes log/, sessions/, tmp/, *.sqlite*, *.lock, model caches)"; snap_ | tee $S/logs/hm-s0.txt
echo; echo "## candidate, flag off (default)"
ex | grep -vE '"ev": "get"'
show run $CAND $H account list | q
show run $CAND $H login status | q
echo; echo "## candidate, --enable named_accounts for single runs: default account"
ex --enable named_accounts | grep -vE '"ev": "get"'
ex --enable named_accounts --account default | grep -vE '"ev": "get"'
ex --enable named_accounts --account openai:default | grep -vE '"ev": "get"'
echo "## snapshot S1 vs S0"; snap_ > $S/logs/hm-s1.txt; diff $S/logs/hm-s0.txt $S/logs/hm-s1.txt && echo "IDENTICAL"
echo; echo "## flag-off parity of named-account commands: BASE vs CAND"
for bin in $BASE $CAND; do
  echo "--- $( [ $bin = $BASE ] && echo BASE || echo CAND )"
  show run $bin $H login --account work --device-auth --experimental_issuer $U < /dev/null | q | tail -3
  printf 'sk-fake-S6B' | show run $bin $H login --account work --with-api-key | q | tail -2
  show run $bin $H logout --account work < /dev/null | q
  show run $bin $H login --account work status | q
  BIN=$bin ex --account work | grep -vE '"ev": "get"'
done
echo "## snapshot S2 vs S0"; snap_ > $S/logs/hm-s2.txt; diff $S/logs/hm-s0.txt $S/logs/hm-s2.txt && echo "IDENTICAL"
echo; echo "## BASE still works on the home afterwards"
BIN=$BASE ex | grep -vE '"ev": "get"'
} > "$EV/captures/carry-flagoff.txt" 2>&1
echo done
