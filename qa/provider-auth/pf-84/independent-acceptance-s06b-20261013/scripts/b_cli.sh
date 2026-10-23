#!/usr/bin/env bash
# Slice B (named ChatGPT/OpenAI sign-in accounts) from the CLI, candidate only, local mock issuer + Responses.
# Fake tokens only. Decoys OPENAI_API_KEY / CODEX_API_KEY / CODEX_ACCESS_TOKEN set for the named-account runs.
# Output: captures/b-cli.txt
set -u
D=$(cd "$(dirname "$0")" && pwd); source "$D/env.sh"; source "$D/common_b.sh"
P=34721; U=http://127.0.0.1:$P; LOG=$S/logs/mock-b1.jsonl; : > "$LOG"
python3 "$D/mock.py" --port $P --log "$LOG" --bearer keyacct=sk-canary-keyacct-S6B --bearer fakekey=sk-fake-S6B \
  --bearer envkey=sk-envdecoy-S6B --bearer codexenv=sk-codexenv-S6B --bearer defkey=sk-canary-defaultkey-S6B &
MOCK=$!; trap 'kill $MOCK' EXIT; sleep 1
export FH=$S/fh-b1 H=$S/homes/b1; [ -d $H ] && rm -r $H < /dev/null; [ -d $FH ] && rm -r $FH < /dev/null; mkdir -p $FH; mkcfg $H
BASEENV=(CODEX_REFRESH_TOKEN_URL_OVERRIDE=$U/oauth/token CORBANU_TEST_LOGIN_ISSUER=$U)
DECOYS=(OPENAI_API_KEY=sk-envdecoy-S6B CODEX_API_KEY=sk-codexenv-S6B
  CODEX_ACCESS_TOKEN=$(python3 -c "import base64,json,time;b=lambda d:base64.urlsafe_b64encode(json.dumps(d).encode()).rstrip(b'=').decode();print(b({'alg':'none'})+'.'+b({'user':'envtoken','gen':0,'exp':int(time.time())+9999})+'.c2ln')"))
{
echo "## candidate $(cat $S/bin/COMMIT-cand); HOME=\$S/fh-b1; CORBANU_HOME=\$S/homes/b1; mock 127.0.0.1:$P"
XENV=("${BASEENV[@]}")
echo; echo "## default login: ChatGPT user deflt (device code), access tokens 30 s"
ctl '{"next_user":"deflt","ttl":30,"refresh_ttl":3600}'
show run $CAND $H login --device-auth --experimental_issuer $U < /dev/null | q
authsha; D0=$(shasum -a 256 $H/auth.json | cut -c1-16)
echo; echo "## B3/B6 named sign-ins (device code), same-user cases"
login work work; login play play; login twin "work~2"; login dup "deflt~2"; login bad bad
printf 'sk-canary-keyacct-S6B' | show run $CAND $H login --account key --with-api-key | q
printf 'sk-fake-S6B' | show run $CAND $H login --account fake --with-api-key | q
authsha
echo; echo "## B5 display"
show run $CAND $H account list openai | q
for a in work twin key bad ghost; do show run $CAND $H login --account $a status | q; done
show run $CAND $H login status | q
echo; echo "## B1 storage: vault labels and files"
find $H -type f -newer $H/config.toml ! -path '*/log/*' | sed "s#$S#\$S#g" | sort
echo "home files holding a named refresh token / key in clear:"; grep -rl -e 'rt.work.' -e 'rt.play.' -e 'sk-canary-keyacct' $H $FH 2>/dev/null | sed "s#$S#\$S#g" || true
echo; echo "## B2 refresh write-back: wait for the 30 s tokens to expire; refreshed tokens live 8 s; decoys set"
ctl '{"refresh_ttl":8}'; sleep 31
XENV=("${BASEENV[@]}" "${DECOYS[@]}")
b=$(mocklines $LOG)
( show run $CAND $H exec --skip-git-repo-check -C $S/ws --account work "Reply with exactly: pong" < /dev/null > $S/logs/b1-par-work.txt ) & w1=$!
( show run $CAND $H exec --skip-git-repo-check -C $S/ws --account play "Reply with exactly: pong" < /dev/null > $S/logs/b1-par-play.txt ) & w2=$!; wait $w1 $w2
echo "(parallel exec on work and play)"; for f in work play; do echo "-- $f:"; grep -h -E '^mock:|^ERROR|\[exit' $S/logs/b1-par-$f.txt | sort | uniq -c; done
echo "-- mock during the parallel runs: refreshes / rejections per user, and the user+gen each /responses carried"
tail -n +$((b+1)) $LOG | python3 -c "import sys,json,collections
c=collections.Counter()
for l in sys.stdin:
  e=json.loads(l)
  if e['ev'] in ('refresh','refresh-rejected'): c[(e['ev'],e['user'],e.get('why',''))]+=1
  if e['ev'].startswith('responses'): c[(e['ev'],e['who'].get('user'),'gen%s'%e['who'].get('gen'))]+=1
for k,v in sorted(c.items()): print('  ',v,'x',*k)"
authsha
echo "(new process on work after its refreshed token expired: gen continues from the vault)"
sleep 9; ex --account work | grep -E 'refresh|responses|exit|^mock' | sed -n '1,4p;$p'
echo "(twin = same ChatGPT user as work, own store)"; ex --account twin | grep -E 'refresh|responses|exit|mock' | head -6
echo; echo "## B9 API-key accounts with env decoys"
ex --account key; ex --account fake
echo; echo "## B8 rejected refresh"; ex --account bad
echo; echo "## C3 missing account"; ex --account ghost; ex --account openai:ghost
authsha; echo "   (D0=$D0)"
echo; echo "## B4 default login unreadable (chmod 000): named session still works"
chmod 000 $H/auth.json; ex --account key; chmod 600 $H/auth.json; authsha
echo; echo "## B3 re-login: another user (old tokens revoked), same user (nothing revoked)"
ctl '{"ttl":3600,"refresh_ttl":3600}'
b=$(mocklines $LOG); login play play2; since $b | grep -E 'revoke|sign-in'
b=$(mocklines $LOG); login play "play2~2"; since $b | grep -E 'revoke|sign-in'
echo; echo "## B3 logout --account play (revokes only play)"
b=$(mocklines $LOG); show run $CAND $H logout --account play < /dev/null | q; since $b
show run $CAND $H logout --account play < /dev/null | q
show run $CAND $H logout --account ghost < /dev/null | q
ex --account play | grep -vE '^   mock: \{"ev": "get"'
show run $CAND $H account list openai | q
authsha
echo; echo "## B3 rename/remove; rename hint when [provider_accounts] names the old name"
printf '[provider_accounts]\nopenai = "twin"\n' >> $H/config.toml
show run $CAND $H account rename openai twin twin2 | q
grep -A1 provider_accounts $H/config.toml
sed -i '' '/^\[provider_accounts\]/,$d' $H/config.toml
ex --account twin2 | grep -E 'responses|exit|ERROR' | head -4
b=$(mocklines $LOG); show run $CAND $H account remove openai twin2 | q; since $b
show run $CAND $H account list openai | q
echo; echo "## default session (decoys set) after all named runs: default login still used and unchanged"
ex | grep -vE '"ev": "get"'
authsha; echo "   (D0=$D0)"
echo; echo "## full mock log (credential-bearing events)"
grep -E '"responses|refresh|revoke|sign-in' "$LOG" | sed 's/^/   /'
} > "$EV/captures/b-cli.txt" 2>&1
echo done
