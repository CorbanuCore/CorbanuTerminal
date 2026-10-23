S=/Volumes/CorbanuDrive/Corbanu/tmp/pf84acc-scratch; B=$S/bin/corbanu-main; P=$S/s02/hP
h() { python3 -c "import hashlib,sys; d=sys.stdin.buffer.read().strip(); print('sha12='+hashlib.sha256(d).hexdigest()[:12] if d else 'EMPTY')"; }
run() { label="$1"; shift; o=$(env -i HOME=$S/fakehome PATH=$S/shim:/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$P "$@" 2>$S/e.txt); rc=$?; echo "$label: exit=$rc stdout=$(printf '%s' "$o" | h) stderr=$(head -c 250 $S/e.txt | tr '\n' ' ')"; }
echo "home hP accounts:"; env -i HOME=$S/fakehome PATH=/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$P $B account list | sed 's/^/  /'
echo "expected: faketok token sha12=$(h < $S/mock/claude-fake-token.txt); cfgacct (.credentials.json in config dir) sha12=$(printf $(printf %s%s sk-ant- oat01-cfgdir-canary-0001) | h)"
echo "-- flag ON (config.toml named_accounts = true)"
run "--account faketok" $B internal-claude-oauth-token --account faketok
run "--account cfgacct" $B internal-claude-oauth-token --account cfgacct
run "--account ghost" $B internal-claude-oauth-token --account ghost
run "CORBANU_PROVIDER_ACCOUNT=faketok" env CORBANU_PROVIDER_ACCOUNT=faketok $B internal-claude-oauth-token
cp $P/config.toml $P/config.toml.on; printf '[features]\nnamed_accounts = false\n' > $P/config.toml
echo "-- flag OFF (config.toml named_accounts = false)  [issue #415]"
run "--account faketok" $B internal-claude-oauth-token --account faketok
run "CORBANU_PROVIDER_ACCOUNT=faketok" env CORBANU_PROVIDER_ACCOUNT=faketok $B internal-claude-oauth-token
cp $P/config.toml.on $P/config.toml
echo "-- env exception stays default-only (CLAUDE_CODE_OAUTH_TOKEN=envtok canary sha12=$(printf envtok-canary-0001 | h))"
run "no account + env token" env CLAUDE_CODE_OAUTH_TOKEN=envtok-canary-0001 $B internal-claude-oauth-token
run "CORBANU_PROVIDER_ACCOUNT=faketok + env token (candidate)" env CLAUDE_CODE_OAUTH_TOKEN=envtok-canary-0001 CORBANU_PROVIDER_ACCOUNT=faketok $B internal-claude-oauth-token
run "CORBANU_PROVIDER_ACCOUNT=faketok + env token (pre-PF-84 build) [issue #414]" env CLAUDE_CODE_OAUTH_TOKEN=envtok-canary-0001 CORBANU_PROVIDER_ACCOUNT=faketok $S/bin/corbanu-pre internal-claude-oauth-token
echo "security(1) CLI shim calls: $(wc -l < $S/shim/security.log | tr -d ' ')"
