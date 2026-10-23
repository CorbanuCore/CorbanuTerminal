S=/Volumes/CorbanuDrive/Corbanu/tmp/pf84acc-scratch; B=$S/bin/corbanu-main; Q=$S/s02/hQ; R=$S/mock/requests.jsonl; AL=$S/mock/authcmd.log
run() { label="$1"; shift; : > $R; : > $AL
  out=$(cd $S/fakehome && env -i HOME=$S/fakehome PATH=/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$Q "$B" exec --skip-git-repo-check "$@" "Reply with exactly: pong" 2>&1); rc=$?
  echo "== $label (exit $rc)"; echo "   args: $*"
  printf '%s\n' "$out" | grep -E 'ERROR|account' | sort -u | head -4 | sed 's/^/   | /'
  echo "   auth.command invocations: $(wc -l < $AL | tr -d ' ') -> $(cut -d' ' -f2 $AL | sort | uniq -c | tr '\n' ';')"
  echo "   mock saw $(wc -l < $R | tr -d ' ') request(s), bearers: $(python3 -c "import json;print(sorted(set(json.loads(l)['bearer_id'] for l in open('$R'))))")"
}
run "Q1 default account"
run "Q2 enrolled named account work" -c provider_accounts.cmdp='"work"'
run "Q3 unenrolled account ghost" -c provider_accounts.cmdp='"ghost"'
run "Q4 flag off + selector work" --disable named_accounts -c provider_accounts.cmdp='"work"'
