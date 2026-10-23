. /Volumes/CorbanuDrive/Corbanu/tmp/pf84acc-scratch/realenv.sh; Z=$S/s02/hZ
run() { label="$1"; shift; out=$(rx $Z "$@" 2>&1); rc=$?; echo "== $label (exit $rc)"; echo "   args: $*"; printf '%s\n' "$out" | summ; }
E=(); run "Z1 named main (real key)" -c provider_accounts.zai='"main"'
E=(); run "Z2 named fake (fake key)" -c provider_accounts.zai='"fake"'
E=(); run "Z3 named ghost (not enrolled)" -c provider_accounts.zai='"ghost"'
E=(); run "Z4 default account, no stored default key and no env" 
# Z5/Z6: real key in ZAI_API_KEY env; named accounts must ignore it
X="$(env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME $HELPER vault auth-helper provider/zai_api_key)" bash -c '. '$S'/realenv.sh; Z=$S/s02/hZ
run() { label="$1"; shift; out=$(rx $Z "$@" 2>&1); rc=$?; echo "== $label (exit $rc)"; echo "   args: $* (env ZAI_API_KEY=<real key, redacted>)"; printf "%s\n" "$out" | summ; }
E=(ZAI_API_KEY="$X"); run "Z5 default account via env (real)"
E=(ZAI_API_KEY="$X"); run "Z6 named fake with REAL key in env: must stay on fake (401)" -c provider_accounts.zai="\"fake\""
E=(ZAI_API_KEY="$X"); run "Z7 named ghost with REAL key in env: must fail closed" -c provider_accounts.zai="\"ghost\""
E=(ZAI_API_KEY="$X"); run "Z8 flag OFF, selector fake, real env key: today = env key works" --disable named_accounts -c provider_accounts.zai="\"fake\""'
