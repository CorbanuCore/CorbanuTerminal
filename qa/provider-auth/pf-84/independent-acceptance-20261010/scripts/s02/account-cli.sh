S=/Volumes/CorbanuDrive/Corbanu/tmp/pf84acc-scratch; B=$S/bin/corbanu-main; N=$S/s02/hN2; mkdir -p $N
cx() { env -i HOME=$S/fakehome PATH=/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$N $B "$@"; }
echo "-- features list: $(cx features list 2>/dev/null | grep named_accounts)"
echo "-- flag OFF (fresh home, no config)"
cx account list 2>&1 | sed 's/^/   /'; printf 'fake-off' | cx account add zai offacct 2>&1 | sed 's/^/   /'; echo "   secrets/ exists after refused add: $([ -d $N/secrets ] && echo yes || echo no)"
printf '[features]\nnamed_accounts = true\n' > $N/config.toml
echo "-- flag ON"
cx account list 2>&1 | sed 's/^/   /'
t() { prov="$1"; shift; printf '%s' "fake-canary-cli" | cx account add "$prov" "$@" >$S/o.txt 2>&1; echo "   [exit $?] account add $prov $* -> $(head -c 200 $S/o.txt | tr '\n' ' ')"; }
echo "-- names"; t zai default; t zai Default; t zai a_b; t zai "a b"; t zai é; t zai ""; t zai -- -lead; t zai aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa; t zai aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa; t zai 0k-1
echo "-- providers / kinds"; t nosuchprov x1; t openai work; t amazon-bedrock work; t claude-plan work; t zai x2 --kind claude-token; t zai x4 --kind command; t claude-plan cfg --kind claude-config-dir
echo "-- secret only from piped stdin"; t zai x3 --value fake-canary-arg
printf '' | cx account add zai empty >$S/o.txt 2>&1; echo "   [exit $?] empty stdin -> $(cat $S/o.txt)"
printf '  \n' | cx account add zai blank >$S/o.txt 2>&1; echo "   [exit $?] whitespace stdin -> $(cat $S/o.txt)"
script -q /dev/null bash -c "env -i HOME=$S/fakehome PATH=/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$N $B account add zai ttyacct </dev/tty; echo \"[exit \$?]\"" < /dev/null 2>&1 | tr -d '\r' | sed 's/^\^D//; s/^/   tty stdin -> /'
echo "-- list / remove"; cx account list | sed 's/^/   /'; cx account remove zai 0k-1 2>&1 | sed 's/^/   /'; cx account remove zai nope 2>&1 | sed 's/^/   /'; cx account list | sed 's/^/   /'
