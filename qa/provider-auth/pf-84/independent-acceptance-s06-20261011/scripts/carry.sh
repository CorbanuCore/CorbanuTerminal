#!/usr/bin/env bash
# Existing single-account home (made by the pre-S06 baseline) carried over; flag-off parity candidate vs baseline.
# Home hm: default ChatGPT login (mock issuer) in auth.json, Bedrock default via ~/.aws default profile.
# Output: captures/carry-flagoff.txt
set -u
D=$(cd "$(dirname "$0")" && pwd); source "$D/env.sh"; source "$D/aws_setup.sh"
P=34641; U=http://127.0.0.1:$P; LOG=$S/logs/mock-c.jsonl; : > "$LOG"
python3 "$D/mock.py" --port $P --log "$LOG" &
MOCK=$!; trap 'kill $MOCK' EXIT; sleep 1
[ -d $S/homes/hm ] && rm -r $S/homes/hm $S/fh-hm < /dev/null
aws_setup hm $P yes
cat >> $H/config.toml <<EOF
openai_base_url = "$U/v1"
chatgpt_base_url = "$U/backend-api/codex"
EOF
sed -i '' '/^\[features\]/d; /^named_accounts = true/d' $H/config.toml
# move the two top-level keys above the tables (TOML) by rebuilding the file
python3 - "$H/config.toml" <<'EOF'
import sys
p=sys.argv[1]; lines=open(p).read().splitlines()
top=[l for l in lines if l.startswith(('openai_base_url','chatgpt_base_url'))]
rest=[l for l in lines if l not in top]
i=next(k for k,l in enumerate(rest) if l.startswith('['))
open(p,'w').write('\n'.join(rest[:i]+top+rest[i:])+'\n')
EOF
OUT=$EV/captures/carry-flagoff.txt
ctl() { curl -s -X POST -H 'content-type: application/json' -d "$1" $U/_ctl > /dev/null; }
XENV=(CODEX_REFRESH_TOKEN_URL_OVERRIDE=$U/oauth/token CORBANU_TEST_LOGIN_ISSUER=$U)
q() { grep -v -E 'WARNING: proceeding|rmcp::transport|^(workdir|approval|sandbox|reasoning|session id|user$|Reply with exactly|warning: Under-dev|tokens used|Reading additional|model:|provider:|corbanu$|129|-{8}|Corbanu Terminal v)|^\s*$' | sed "s#$CAND#\$CAND#g; s#$BASE#\$BASE#g; s#$S#\$S#g"; }
ex() { local bin=$1; shift; local b=$(mocklines $LOG); show run $bin $H exec --skip-git-repo-check -C $S/ws "$@" "Reply with exactly: pong" < /dev/null | q
  tail -n +$((b+1)) $LOG | grep -E '"responses|refresh|revoke' | sed 's/^/   mock: /' | cut -c1-200; }
snap() { (cd $H && find . -type f ! -path './log/*' ! -path './sessions/*' ! -path './tmp/*' ! -name '*.sqlite*' ! -name '*.lock' -print0 | sort -z | xargs -0 shasum -a 256 | cut -c1-16,65-); shasum -a 256 $FH/.aws/config $FH/.aws/credentials | cut -c1-16,65- | sed "s#$S#\$S#g"; }
{
echo "## home hm made by the BASELINE (3a0a38bed2): default ChatGPT login + Bedrock default (~/.aws [default]); no features table"
ctl '{"next_user":"legacy","ttl":3600,"refresh_ttl":3600}'
run $BASE $H login --device-auth --experimental_issuer $U < /dev/null 2>&1 | tail -1
ex $BASE -c 'model_provider="openai"' -m gpt-5.4
ex $BASE
echo "## snapshot S0 (sha256 prefixes of every non-log file + ~/.aws)"; snap | tee $S/logs/hm-s0.txt
echo; echo "## candidate, flag off (default): same credentials as the baseline?"
ex $CAND -c 'model_provider="openai"' -m gpt-5.4
ex $CAND
show run $CAND $H account list | q
show run $CAND $H login status | q
show run $CAND $H login --account x status | q
echo; echo "## candidate, flag ON for this run only (--enable), default account"
ex $CAND --enable named_accounts -c 'model_provider="openai"' -m gpt-5.4
ex $CAND --enable named_accounts
ex $CAND --enable named_accounts --account default
echo "## snapshot S1 and diff vs S0"; snap > $S/logs/hm-s1.txt; diff $S/logs/hm-s0.txt $S/logs/hm-s1.txt && echo "IDENTICAL (auth.json, config.toml, vault files, ~/.aws)"
echo; echo "## flag-off parity on named-account commands: baseline vs candidate"
for bin in $BASE $CAND; do
  echo "--- $( [ $bin = $BASE ] && echo BASE || echo CAND )"
  show run $bin $H account add amazon-bedrock work --kind aws-profile --value work | q
  show run $bin $H login --account work --device-auth --experimental_issuer $U < /dev/null | q | tail -3
  show run $bin $H logout --account work < /dev/null | q
  ex $bin -c 'model_provider="amazon-bedrock"' --account work
done
echo "## snapshot S2 and diff vs S0"; snap > $S/logs/hm-s2.txt; diff $S/logs/hm-s0.txt $S/logs/hm-s2.txt && echo "IDENTICAL"
echo; echo "## the baseline still works on the home afterwards"
ex $BASE -c 'model_provider="openai"' -m gpt-5.4
} > "$OUT" 2>&1
echo done
