#!/bin/bash
# PF-84 #414/#419.3 functional check. Never prints a token.
S=<scratch>
HC=$S/hc; rm -r "$HC" 2>/dev/null </dev/null; mkdir -p "$HC"
printf '[features]\nnamed_accounts = true\n' > "$HC/config.toml"
run() { # parent pathdir args...
  local parent=$1 pathdir=$2; shift 2
  (cd $S/work && env -i HOME=$S/fakehome PATH=$S/$pathdir:/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$HC \
    CLAUDE_CODE_OAUTH_TOKEN="$CLAUDE_CODE_OAUTH_TOKEN" $S/bin/corbanu-$parent "$@")
}
H="$S/bin/corbanu-after"
printf 'sk-ant-''oat01-pf84fix-fake-not-a-token' | env -i HOME=$S/fakehome PATH=/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$HC $H account add claude-plan fake --kind claude-token
printf '%s' "$CLAUDE_CODE_OAUTH_TOKEN" | env -i HOME=$S/fakehome PATH=/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$HC $H account add claude-plan real --kind claude-token
env -i HOME=$S/fakehome PATH=/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$HC $H account list
P="Reply with exactly the word pong and nothing else."
for c in "before path-pre fake" "after path-pre fake" "after path-after fake" "after path-after real" "before path-after ghost" "after path-after ghost"; do
  set -- $c
  : > $S/$2/calls.log
  out=$(run $1 $2 exec --skip-git-repo-check -m claude-opus-5-5-plan -c model_provider='"claude-plan"' -c provider_accounts.claude-plan="\"$3\"" "$P" 2>&1); rc=$?
  echo "== parent=$1 PATH-helper=${2#path-} account=$3 (default token in CLAUDE_CODE_OAUTH_TOKEN) exit=$rc helper-calls=$(wc -l < $S/$2/calls.log | tr -d ' ')"
  printf '%s\n' "$out" | grep -E '^pong|^ERROR' | sort -u | head -3 | cut -c1-400 | sed 's/^/   | /'
  head -1 $S/$2/calls.log | sed 's/^/   first call: /'
done
echo DONE-414
