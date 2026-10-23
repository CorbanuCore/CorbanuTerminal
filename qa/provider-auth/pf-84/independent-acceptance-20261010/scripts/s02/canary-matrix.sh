S=/Volumes/CorbanuDrive/Corbanu/tmp/pf84acc-scratch; B=$S/bin/corbanu-main; M=$S/s02/hM; R=$S/mock/requests.jsonl
ENVC=$(python3 -c "import json;d=json.load(open('$S/mock/canaries.json'));print(next(k for k,v in d.items() if v=='env'))")
run() { label="$1"; shift; : > $R
  out=$(cd $S/fakehome && env -i HOME=$S/fakehome PATH=/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$M MOCK_API_KEY="$ENVC" $B exec --skip-git-repo-check "$@" "Reply with exactly: pong" 2>&1); rc=$?
  echo "== $label (exit $rc)"; echo "   args: $*"
  printf '%s\n' "$out" | grep -i -E 'error|401|account|warn|unauthor|pong' | grep -v -E '^\s*$' | head -6 | sed 's/^/   | /'
  echo "   mock saw $(wc -l < $R | tr -d ' ') request(s): $(python3 -c "import json;print(sorted(set(json.loads(l)['bearer_id'] for l in open('$R'))))")"
  python3 - "$out" <<PY
import json,sys; d=json.load(open("$S/mock/canaries.json")); o=sys.argv[1]
print("   canary values in output:", [v for k,v in d.items() if k in o] or "none")
PY
}
run "C1 default account (env MOCK_API_KEY set)"
run "C2 -c provider_accounts.mockp=acct-a, env also set" -c provider_accounts.mockp='"acct-a"'
run "C3 -c provider_accounts.mockp=acct-b, env also set" -c provider_accounts.mockp='"acct-b"'
run "C4 unknown account ghost, env also set" -c provider_accounts.mockp='"ghost"'
run "C5 invalid account name Bad_Name" -c provider_accounts.mockp='"Bad_Name"'
run "C6 explicit default selector" -c provider_accounts.mockp='"default"'
run "C7 flag OFF + selector acct-a (expect today's behaviour: env key)" --disable named_accounts -c provider_accounts.mockp='"acct-a"'
