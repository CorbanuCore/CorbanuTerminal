#!/usr/bin/env bash
# Slice B (named ChatGPT/OpenAI sign-in accounts) from the CLI against the local mock issuer + Responses.
# Fake tokens only (issued by mock.py). Decoy OPENAI_API_KEY / CODEX_API_KEY / CODEX_ACCESS_TOKEN set for
# every named-account run. Output: captures/oa-cli.txt
set -u
D=$(cd "$(dirname "$0")" && pwd); source "$D/env.sh"
P=34621; U=http://127.0.0.1:$P; LOG=$S/logs/mock-b.jsonl; : > "$LOG"
python3 "$D/mock.py" --port $P --log "$LOG" --bearer keyacct=sk-canary-keyacct-S06CANARY \
  --bearer fakekey=sk-fake-S06CANARY --bearer envkey=sk-envdecoy-S06CANARY --bearer codexenv=sk-codexenv-S06CANARY &
MOCK=$!; trap 'kill $MOCK' EXIT; sleep 1
export FH=$S/fh-b1 H=$S/homes/b1
[ -d $H ] && rm -r $H $FH < /dev/null; mkdir -p $H $FH
cat > $H/config.toml <<EOF
model = "gpt-5.4"
model_provider = "openai"
openai_base_url = "$U/v1"
chatgpt_base_url = "$U/backend-api/codex"
[features]
named_accounts = true
[projects."$S/ws"]
trust_level = "trusted"
EOF
OUT=$EV/captures/oa-cli.txt
ctl() { curl -s -X POST -H 'content-type: application/json' -d "$1" $U/_ctl > /dev/null; }
BASEENV=(CODEX_REFRESH_TOKEN_URL_OVERRIDE=$U/oauth/token CORBANU_TEST_LOGIN_ISSUER=$U)
DECOYS=(OPENAI_API_KEY=sk-envdecoy-S06CANARY CODEX_API_KEY=sk-codexenv-S06CANARY
  CODEX_ACCESS_TOKEN=$(python3 -c "import base64,json,time;b=lambda d:base64.urlsafe_b64encode(json.dumps(d).encode()).rstrip(b'=').decode();print(b({'alg':'none'})+'.'+b({'user':'envtoken','gen':0,'exp':int(time.time())+9999})+'.c2ln')"))
login() { ctl "{\"next_user\":\"$2\"}"; show run $CAND $H login --account $1 --device-auth --experimental_issuer $U < /dev/null; }
ex() { show run $CAND $H exec --skip-git-repo-check -C $S/ws "$@" "Reply with exactly: pong" < /dev/null; }
cnt() { local b=$(mocklines $LOG); ex "$@"; echo "-- mock events during this run:"; tail -n +$((b+1)) $LOG | python3 -c "import sys,json
for l in sys.stdin:
  e=json.loads(l); e.pop('t'); print('   ', json.dumps(e))"; }
authsha() { echo "auth.json: $( [ -f $H/auth.json ] && shasum -a 256 $H/auth.json | cut -c1-16 || echo absent)"; }
{
echo "## candidate a22fdb903d; HOME=\$S/fh-b1; CORBANU_HOME=\$S/homes/b1; mock at 127.0.0.1:$P"
XENV=("${BASEENV[@]}")
echo; echo "## default login (ChatGPT user deflt, access token expires in 30 s)"
ctl '{"next_user":"deflt","ttl":30}'
show run $CAND $H login --device-auth --experimental_issuer $U < /dev/null
authsha; ls -la $H | sed "s#$S#\$S#g" | grep -v '^total'
D0=$(shasum -a 256 $H/auth.json | cut -c1-16)
echo; echo "## B3 named sign-ins (device code); same-user cases"
login work work; authsha
login play play; authsha
login twin "work~2"
login dup "deflt~2"
login bad bad
printf 'sk-canary-keyacct-S06CANARY' | show run $CAND $H login --account key --with-api-key
printf 'sk-fake-S06CANARY' | show run $CAND $H login --account fake --with-api-key
echo "(re-login of an existing account to another user: allowed? replaced tokens revoked?)"
login play2 play2
authsha
echo; echo "## B5 display"
show run $CAND $H account list openai
show run $CAND $H account list
show run $CAND $H login status
for a in work twin key bad ghost; do show run $CAND $H login --account $a status; done
echo; echo "## B1 storage: per-account vault labels, no extra files"
echo "(the vault values cannot be read back: the candidate's auth-helper refuses provider-integration labels)"
show run $CAND $H vault auth-helper provider/openai/accounts/work/chatgpt_auth
find $H -type f -newer $H/config.toml | sed "s#$S#\$S#g" | sort
echo "files under the homes holding a named account's refresh token in clear:"; grep -rl -e 'rt.work.' -e 'rt.play.' -e 'sk-canary-keyacct' $H $FH 2>/dev/null | sed "s#$S#\$S#g" || true
echo; echo "## B2 refresh write-back: wait for the 30 s access tokens to expire; refreshed tokens live 8 s"
ctl '{"refresh_ttl":8}'; sleep 31
XENV=("${BASEENV[@]}" "${DECOYS[@]}")
b=$(mocklines $LOG)
( ex --account work > $S/logs/b-par-work.txt ) & w1=$!; ( ex --account play > $S/logs/b-par-play.txt ) & w2=$!; wait $w1 $w2
echo "(parallel exec on work and play)"; grep -h -E '^mock:|ERROR|rror' $S/logs/b-par-work.txt $S/logs/b-par-play.txt | sort -u
tail -n +$((b+1)) $LOG | python3 -c "import sys,json
for l in sys.stdin:
  e=json.loads(l); e.pop('t'); print('   ', json.dumps(e))"
authsha
echo "(a NEW process on work after its refreshed token expired: must present gen 2 from the vault)"
sleep 9; cnt --account work; authsha
echo "(twin = same ChatGPT user as work, own store: still gen 1)"
cnt --account twin
echo; echo "## B9 API-key account and env decoys"
cnt --account key
cnt --account fake
echo; echo "## B8 rejected refresh"
cnt --account bad
echo; echo "## C3 missing account"
cnt --account ghost
cnt --account openai:ghost
authsha; echo "(D0=$D0)"
echo; echo "## B4 default login unreadable (chmod 000 auth.json): named account still works?"
chmod 000 $H/auth.json; cnt --account work; chmod 600 $H/auth.json; authsha
echo; echo "## B3 logout --account play2 (revokes only play2)"
show run $CAND $H logout --account play2 < /dev/null
b=$(mocklines $LOG); tail -n 3 $LOG
show run $CAND $H account list openai
show run $CAND $H logout --account play2 < /dev/null
show run $CAND $H logout --account ghost < /dev/null
cnt --account play2
cnt --account work
authsha
echo; echo "## B7 restrictions"
cnt --account work -c 'forced_login_method="api"'
cnt --account key -c 'forced_login_method="api"'
cnt --account key -c 'forced_login_method="chatgpt"'
cnt --account work -c 'forced_chatgpt_workspace_id="ws-other"'
cnt --account work -c 'forced_chatgpt_workspace_id="ws-main"'
echo; echo "## flag off"
show run $CAND $H login --account flagoff --device-auth --experimental_issuer $U --disable named_accounts < /dev/null
show run $CAND $H logout --account work --disable named_accounts < /dev/null
cnt --account work --disable named_accounts
authsha; echo "(D0=$D0)"
echo; echo "## default session (decoys still set) after all named runs"
cnt
authsha
echo; echo "## full mock log"
sed "s#$S#\$S#g" "$LOG"
} > "$OUT" 2>&1
echo done
