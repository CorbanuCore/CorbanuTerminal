#!/usr/bin/env bash
# #570 re-verification: a session on a NAMED OpenAI account under forced_login_method / forced_chatgpt_workspace_id
# that rule out the DEFAULT login. Fresh home per case; candidate vs BASE (main before the fix). Output: captures/b-forced.txt
set -u
D=$(cd "$(dirname "$0")" && pwd); source "$D/env.sh"; source "$D/common_b.sh"
P=34723; U=http://127.0.0.1:$P; LOG=$S/logs/mock-b4.jsonl; : > "$LOG"
python3 "$D/mock.py" --port $P --log "$LOG" --bearer keyacct=sk-canary-keyacct-S6B --bearer defkey=sk-canary-defaultkey-S6B \
  --bearer codexenv=sk-codexenv-S6B --bearer envkey=sk-envdecoy-S6B &
MOCK=$!; trap 'kill $MOCK' EXIT; sleep 1
export FH=$S/fh-b4; mkdir -p $FH
XENV=(CODEX_REFRESH_TOKEN_URL_OVERRIDE=$U/oauth/token CORBANU_TEST_LOGIN_ISSUER=$U)
mkhome() { # mkhome <name> <default: key|chatgpt>
  H=$S/homes/$1; [ -d $H ] && rm -r $H < /dev/null; mkcfg $H
  if [ $2 = key ]; then printf 'sk-canary-defaultkey-S6B' | run $CAND $H login --with-api-key > /dev/null 2>&1
  else ctl '{"next_user":"deflt","ttl":3600,"refresh_ttl":3600}'; run $CAND $H login --device-auth --experimental_issuer $U < /dev/null > /dev/null 2>&1; fi
  ctl '{"next_user":"named","ttl":3600}'; run $CAND $H login --account chat --device-auth --experimental_issuer $U < /dev/null > /dev/null 2>&1
  ctl '{"next_user":"namedo@ws=other","ttl":3600}'; run $CAND $H login --account chato --device-auth --experimental_issuer $U < /dev/null > /dev/null 2>&1
  printf 'sk-canary-keyacct-S6B' | run $CAND $H login --account key --with-api-key > /dev/null 2>&1
}
case_() { # case_ <label> <home> <default kind> <bin> <args...>
  local label=$1 name=$2 kind=$3 bin=$4; shift 4
  echo; echo "### $label"; mkhome $name $kind; authsha
  BIN=$bin ex "$@" | grep -vE '"ev": "get"'
  authsha
}
{
echo "## CAND $(cat $S/bin/COMMIT-cand); BASE $(cat $S/bin/COMMIT-base) (main before #604)"
case_ "1 CAND default=API key, forced chatgpt, --account chat (named ChatGPT: allowed)" f1 key $CAND -c 'forced_login_method="chatgpt"' --account chat
case_ "1b BASE same" f1b key $BASE -c 'forced_login_method="chatgpt"' --account chat
case_ "2 CAND default=API key, forced chatgpt, --account key (named API key: refused)" f2 key $CAND -c 'forced_login_method="chatgpt"' --account key
case_ "2b BASE same" f2b key $BASE -c 'forced_login_method="chatgpt"' --account key
case_ "3 CAND default=API key, forced chatgpt, default session (old enforcement expected)" f3 key $CAND -c 'forced_login_method="chatgpt"'
case_ "5 CAND default=ChatGPT, forced api, --account key (allowed)" f5 chatgpt $CAND -c 'forced_login_method="api"' --account key
case_ "5b BASE same" f5b chatgpt $BASE -c 'forced_login_method="api"' --account key
case_ "6 CAND default=ChatGPT, forced api, --account chat (named ChatGPT: refused)" f6 chatgpt $CAND -c 'forced_login_method="api"' --account chat
case_ "7 CAND default=ChatGPT ws-main, forced ws-other, --account key" f7 chatgpt $CAND -c 'forced_chatgpt_workspace_id="ws-other"' --account key
case_ "7b BASE same" f7b chatgpt $BASE -c 'forced_chatgpt_workspace_id="ws-other"' --account key
case_ "8 CAND default=ChatGPT ws-main, forced ws-other, --account chato (ws-other: allowed)" f8 chatgpt $CAND -c 'forced_chatgpt_workspace_id="ws-other"' --account chato
case_ "9 CAND default=ChatGPT ws-main, forced ws-other, --account chat (ws-main: refused)" f9 chatgpt $CAND -c 'forced_chatgpt_workspace_id="ws-other"' --account chat
echo; echo "### 10 CAND default=API key, forced chatgpt, --account chat with CODEX_API_KEY and OPENAI_API_KEY decoys"
mkhome f10 key; authsha
XENV+=(CODEX_API_KEY=sk-codexenv-S6B OPENAI_API_KEY=sk-envdecoy-S6B)
ex -c 'forced_login_method="chatgpt"' --account chat | grep -vE '"ev": "get"'; authsha
echo "### 10b same, --account key (named API key under forced chatgpt, decoys set): refused by the named check, not 'Logging out'"
ex -c 'forced_login_method="chatgpt"' --account key | grep -vE '"ev": "get"'; authsha
XENV=("${XENV[@]:0:2}")
echo; echo "### 11 CAND exec resume of a named thread under forced chatgpt (default=API key)"
mkhome f11 key
run $CAND $H exec --json --skip-git-repo-check -C $S/ws --account chat "Reply with exactly: pong" < /dev/null > $S/logs/f11.jsonl 2>/dev/null
TID=$(python3 -c "import json
for l in open('$S/logs/f11.jsonl'):
  try: e=json.loads(l)
  except ValueError: continue
  if e.get('type')=='thread.started': print(e['thread_id']); break")
echo "thread: $TID"; authsha
ex -c 'forced_login_method="chatgpt"' resume $TID | grep -vE '"ev": "get"'; authsha
echo "### 11b the same thread resumed with no restriction (control)"
ex resume $TID | grep -vE '"ev": "get"'; authsha
echo; echo "### 12 CAND login status / account list under the restriction (default=API key, forced chatgpt)"
show run $CAND $H -c 'forced_login_method="chatgpt"' login status | q
show run $CAND $H -c 'forced_login_method="chatgpt"' account list openai | q
authsha
} > "$EV/captures/b-forced.txt" 2>&1
echo done
