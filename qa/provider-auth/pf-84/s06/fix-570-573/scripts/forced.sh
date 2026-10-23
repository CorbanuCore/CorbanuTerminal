#!/usr/bin/env bash
# #570: does a session on a NAMED OpenAI account log out (delete) the DEFAULT login when
# forced_login_method / forced_chatgpt_workspace_id rule the default out? BASE (origin/main) vs CAND.
# Fresh disposable home per case; local mock issuer + Responses; fake tokens only.
set -u
source ~/corbanu-rtx/f57x-qa/env.sh
P=34742; export U=http://127.0.0.1:$P; LOG=$S/logs/mock-forced.jsonl; : > "$LOG"
python3 $Q/mock.py --port $P --log "$LOG" --bearer keyacct=sk-canary-keyacct-570 --bearer defkey=sk-canary-defaultkey-570 &
MOCK=$!; trap 'kill $MOCK' EXIT; sleep 1
export FH=$S/fh-forced; mkdir -p $FH
XENV=(CODEX_REFRESH_TOKEN_URL_OVERRIDE=$U/oauth/token CORBANU_TEST_LOGIN_ISSUER=$U)
q() { grep -E '^(mock:|Error|Login is|API key login|ChatGPT login|The default|account `)|^\[exit' | sort -u | sed "s#$S#\$S#g"; }
mkhome() { # mkhome <name> <default: key|chatgpt>
  H=$S/homes/$1; [ -d $H ] && rm -r $H < /dev/null; mkdir -p $H
  printf 'model = "gpt-5.4"\nmodel_provider = "openai"\nopenai_base_url = "%s/v1"\nchatgpt_base_url = "%s/backend-api/codex"\nsuppress_unstable_features_warning = true\n[features]\nnamed_accounts = true\n[projects."%s/ws"]\ntrust_level = "trusted"\n' $U $U $S > $H/config.toml
  if [ $2 = key ]; then printf 'sk-canary-defaultkey-570' | run $BASE $H login --with-api-key > /dev/null 2>&1
  else ctl '{"next_user":"deflt","ttl":3600}'; run $BASE $H login --device-auth --experimental_issuer $U < /dev/null > /dev/null 2>&1; fi
  ctl '{"next_user":"named","ttl":3600}'; run $BASE $H login --account chat --device-auth --experimental_issuer $U < /dev/null > /dev/null 2>&1
  printf 'sk-canary-keyacct-570' | run $BASE $H login --account key --with-api-key > /dev/null 2>&1
}
case_() { # case_ <label> <home> <default kind> <bin> <args...>
  local label=$1 name=$2 kind=$3 bin=$4; shift 4
  echo; echo "### $label"; mkhome $name $kind
  local before; before=$(sha256sum $H/auth.json | cut -c1-12)
  echo "   default login before: auth.json $before"
  local b=$(mocklines $LOG)
  show run $bin $H exec --skip-git-repo-check -C $S/ws "$@" "Reply with exactly: pong" < /dev/null | q
  if [ -f $H/auth.json ]; then a=$(sha256sum $H/auth.json | cut -c1-12); echo "   default login after:  auth.json $a $([ $a = $before ] && echo '(unchanged)' || echo '(CHANGED)')"; else echo "   default login after:  auth.json ABSENT (logged out)"; fi
  tail -n +$((b+1)) $LOG | grep -E '"responses' | sed 's/^/   mock: /'
}
for B in BASE CAND; do
  bin=${!B}
  echo; echo "======== $B ($(basename $bin))"
  case_ "1 $B, default=API key, forced chatgpt, --account chat (named ChatGPT)" f1$B key $bin -c 'forced_login_method="chatgpt"' --account chat
  case_ "5 $B, default=ChatGPT, forced api, --account key (named API key)" f5$B chatgpt $bin -c 'forced_login_method="api"' --account key
  case_ "7 $B, default=ChatGPT (ws-main), forced workspace ws-other, --account key" f7$B chatgpt $bin -c 'forced_chatgpt_workspace_id="ws-other"' --account key
  case_ "3 $B, default=API key, forced chatgpt, default session (unchanged behaviour expected)" f3$B key $bin -c 'forced_login_method="chatgpt"'
done
