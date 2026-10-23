#!/usr/bin/env bash
# Slice B, third pass on home b3: rename and remove of a named ChatGPT account (remove revokes only it).
# Output: captures/oa-cli3.txt
set -u
D=$(cd "$(dirname "$0")" && pwd); source "$D/env.sh"
P=34622; U=http://127.0.0.1:$P; LOG=$S/logs/mock-b5.jsonl; : > "$LOG"   # same port as oa_cli2.sh (base URLs in b3's config)
python3 "$D/mock.py" --port $P --log "$LOG" &
MOCK=$!; trap 'kill $MOCK' EXIT; sleep 1
export FH=$S/fh-b3 H=$S/homes/b3; OUT=$EV/captures/oa-cli3.txt
XENV=(CODEX_REFRESH_TOKEN_URL_OVERRIDE=$U/oauth/token CORBANU_TEST_LOGIN_ISSUER=$U)
q() { grep -v -E 'WARNING: proceeding|rmcp::transport|^(workdir|approval|sandbox|reasoning|session id|user$|Reply with exactly|warning: Under-dev|tokens used|Reading additional|model:|provider:|corbanu$|129|-{8}|Corbanu Terminal v)|^\s*$' | sed "s#$CAND#\$CAND#g; s#$S#\$S#g"; }
ex() { local b=$(mocklines $LOG); show run $CAND $H exec --skip-git-repo-check -C $S/ws "$@" "Reply with exactly: pong" < /dev/null | q
  tail -n +$((b+1)) $LOG | grep -E '"responses|refresh|revoke' | sed 's/^/   mock: /' | cut -c1-200; }
{
show run $CAND $H account list openai | q
show run $CAND $H account rename openai r2 r2b | q
ex --account r2b
ex --account r2
b=$(mocklines $LOG)
show run $CAND $H account remove openai r2b | q
echo "-- mock during remove:"; tail -n +$((b+1)) $LOG | sed 's/^/   /'
show run $CAND $H account list openai | q
ex --account r2b
} > "$OUT" 2>&1
echo done
