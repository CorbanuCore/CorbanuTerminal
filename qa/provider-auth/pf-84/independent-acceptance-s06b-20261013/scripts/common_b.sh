# Source after env.sh. Helpers for the slice-B CLI scripts. Needs U (mock URL), LOG (mock log), H (home).
ctl() { curl -s -X POST -H 'content-type: application/json' -d "$1" $U/_ctl > /dev/null; }
mkcfg() { # mkcfg <home> [provider]
  mkdir -p $1; printf 'model = "%s"\nmodel_provider = "%s"\nopenai_base_url = "%s/v1"\nchatgpt_base_url = "%s/backend-api/codex"\n[features]\nnamed_accounts = true\n[projects."%s/ws"]\ntrust_level = "trusted"\n' \
    "$( [ "${2:-openai}" = zai ] && echo glm-5.3-flash || echo gpt-5.4)" "${2:-openai}" $U $U $S > $1/config.toml; }
q() { grep -v -E 'WARNING: proceeding|Welcome|command-line|Follow these|Open this|codex/device|one-time code|MOCK-0001|Continue only|rmcp::transport|^(workdir|approval|sandbox|reasoning|session id|user$|Reply with exactly|warning: Under-dev|tokens used|Reading additional|model:|provider:|corbanu$|129|-{8}|Corbanu Terminal v)|^\s*$' | sed "s#$S#\$S#g"; }
login() { ctl "{\"next_user\":\"$2\"}"; local a=$1; shift 2; show run ${BIN:-$CAND} $H login --account $a --device-auth --experimental_issuer $U "$@" < /dev/null | q; }
since() { tail -n +$(($1+1)) $LOG | python3 -c "import sys,json
for l in sys.stdin:
  e=json.loads(l); e.pop('t')
  if e['ev']=='post-unhandled' or (e['ev']=='get' and e.get('who',{}).get('scheme')=='none'): continue
  print('   mock:', json.dumps(e))"; }
ex() { local b=$(mocklines $LOG); show run ${BIN:-$CAND} $H exec --skip-git-repo-check -C $S/ws "$@" "Reply with exactly: pong" < /dev/null | q; since $b; }
authsha() { echo "   default auth.json: $( [ -f $H/auth.json ] && shasum -a 256 $H/auth.json | cut -c1-16 || echo ABSENT)"; }
