#!/usr/bin/env bash
# Does a session on a NAMED OpenAI account log out the DEFAULT login when forced_login_method rules the
# default out? Fresh home per case; candidate vs pre-S06 baseline. Output: captures/oa-forced.txt
set -u
D=$(cd "$(dirname "$0")" && pwd); source "$D/env.sh"
P=34623; U=http://127.0.0.1:$P; LOG=$S/logs/mock-b4.jsonl; : > "$LOG"
python3 "$D/mock.py" --port $P --log "$LOG" --bearer keyacct=sk-canary-keyacct-S06CANARY --bearer defkey=sk-canary-defaultkey-S06CANARY &
MOCK=$!; trap 'kill $MOCK' EXIT; sleep 1
export FH=$S/fh-b4; mkdir -p $FH
OUT=$EV/captures/oa-forced.txt
ctl() { curl -s -X POST -H 'content-type: application/json' -d "$1" $U/_ctl > /dev/null; }
XENV=(CODEX_REFRESH_TOKEN_URL_OVERRIDE=$U/oauth/token CORBANU_TEST_LOGIN_ISSUER=$U)
q() { grep -v -E 'WARNING: proceeding|Welcome|command-line|Follow these|Open this|codex/device|one-time code|MOCK-0001|Continue only|rmcp::transport|^(workdir|approval|sandbox|reasoning|session id|user$|Reply with exactly|warning: Under-dev|tokens used|Reading additional|model:|provider:|corbanu$|129|-{8}|Corbanu Terminal v)|^\s*$' | sed "s#$CAND#\$CAND#g; s#$BASE#\$BASE#g"; }
# mkhome <name> <default: key|chatgpt>
mkhome() {
  H=$S/homes/$1; [ -d $H ] && rm -r $H < /dev/null; mkdir -p $H
  printf 'model = "gpt-5.4"\nmodel_provider = "openai"\nopenai_base_url = "%s/v1"\nchatgpt_base_url = "%s/backend-api/codex"\n[features]\nnamed_accounts = true\n[projects."%s/ws"]\ntrust_level = "trusted"\n' $U $U $S > $H/config.toml
  if [ $2 = key ]; then printf 'sk-canary-defaultkey-S06CANARY' | run $CAND $H login --with-api-key > /dev/null 2>&1
  else ctl '{"next_user":"deflt","ttl":3600}'; run $CAND $H login --device-auth --experimental_issuer $U < /dev/null > /dev/null 2>&1; fi
  ctl '{"next_user":"named","ttl":3600}'; run $CAND $H login --account chat --device-auth --experimental_issuer $U < /dev/null > /dev/null 2>&1
  printf 'sk-canary-keyacct-S06CANARY' | run $CAND $H login --account key --with-api-key > /dev/null 2>&1
}
st() { echo "   default login after: $( [ -f $H/auth.json ] && echo "auth.json present $(shasum -a 256 $H/auth.json | cut -c1-12)" || echo 'auth.json ABSENT')"; }
case_() { # case_ <label> <home> <default kind> <bin> <args...>
  local label=$1 name=$2 kind=$3 bin=$4; shift 4
  echo; echo "### $label"; mkhome $name $kind
  echo "   default login before: auth.json $(shasum -a 256 $H/auth.json | cut -c1-12)"
  local b=$(mocklines $LOG)
  show run $bin $H exec --skip-git-repo-check -C $S/ws "$@" "Reply with exactly: pong" < /dev/null | q
  st; tail -n +$((b+1)) $LOG | grep -E '"responses|revoke' | sed 's/^/   mock: /'
}
{
case_ "1 CAND, default=API key, forced chatgpt, --account chat (named ChatGPT: allowed)" f1 key $CAND -c 'forced_login_method="chatgpt"' --account chat
case_ "2 CAND, default=API key, forced chatgpt, --account key (named API key: refused)" f2 key $CAND -c 'forced_login_method="chatgpt"' --account key
case_ "3 CAND, default=API key, forced chatgpt, default session" f3 key $CAND -c 'forced_login_method="chatgpt"'
case_ "4 BASE (pre-S06), default=API key, forced chatgpt, default session" f4 key $BASE -c 'forced_login_method="chatgpt"'
case_ "5 CAND, default=ChatGPT, forced api, --account key (named API key: allowed)" f5 chatgpt $CAND -c 'forced_login_method="api"' --account key
case_ "6 BASE (pre-S06), default=ChatGPT, forced api, default session" f6 chatgpt $BASE -c 'forced_login_method="api"'
case_ "7 CAND, default=ChatGPT (ws-main), forced workspace ws-other, --account key" f7 chatgpt $CAND -c 'forced_chatgpt_workspace_id="ws-other"' --account key
} > "$OUT" 2>&1
echo done
