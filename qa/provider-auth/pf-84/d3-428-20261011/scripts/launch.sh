#!/bin/bash
# usage: launch.sh <bin> <rundir>
# Disposable home with features named_accounts + security_levels (+ EXTRA_FEATURES), a named Z.AI
# account `work` holding a fake key, and the real default Z.AI key from the vault helper (never shown).
# MODE=unverified pre-writes security_level.toml = Aggressive (saved without a preflight).
BIN=$1; R=$2; REAL_HOME=$HOME
mkdir -p $R/home $R/ws $R/userhome $R/logs
cd $R/ws; git init -q . 2>/dev/null
cat > $R/home/config.toml <<CFG
suppress_unstable_features_warning = true
[features]
named_accounts = true
security_levels = true
${EXTRA_FEATURES}
[projects."$R/ws"]
trust_level = "trusted"
CFG
[ "$MODE" = unverified ] && printf 'version = 1\nlevel = "aggressive"\n' > $R/home/security_level.toml
for v in $(compgen -e); do case "$v" in *API_KEY*|*TOKEN*|*SECRET*|*PASSWORD*|*CREDENTIAL*|*_AUTH*) unset "$v";; esac; done
export HOME=$R/userhome CODEX_HOME=$R/home CORBANU_HOME=$R/home PFTERMINAL_HOME=$R/home CORBANU_TEST_NO_NATIVE_KEYRING=1 TERM=xterm-256color
printf pf84-fake-zai-key-0428 | $BIN account add zai work > $R/logs/account-add.log 2>&1
ZAI_API_KEY="$(env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME HOME=$REAL_HOME $REAL_HOME/.local/bin/corbanu vault auth-helper provider/zai_api_key)" exec $BIN -C $R/ws -c log_dir=\"$R/logs\" -m glm-5.3-flash -c model_provider=\"zai\"
