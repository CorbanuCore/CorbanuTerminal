#!/bin/bash
# usage: launch.sh <bin> <rundir>
# Disposable home with the "Aggressive ships" feature flags; the real Z.AI key and the Claude plan test
# token come from the installed vault helper on the consuming command only (never shown or written).
# Claude Code gets a disposable config folder.
BIN=$1; R=$2; REAL_HOME=$HOME
mkdir -p $R/home $R/ws $R/userhome $R/logs $R/claude-config
cd $R/ws; git init -q . 2>/dev/null; printf 'buy coffee\n' > notes.txt
cat > $R/home/config.toml <<CFG
suppress_unstable_features_warning = true
[features]
security_levels = true
protected_mode_preflight = true
secretless_agent_launch = true
isolated_credential_broker = true
secret_output_gate = true
source_envelopes = true
url_destination_policy = true
contained_external_agents = ${CONTAINED:-true}
[projects."$R/ws"]
trust_level = "trusted"
CFG
for v in $(compgen -e); do case "$v" in *API_KEY*|*TOKEN*|*SECRET*|*PASSWORD*|*CREDENTIAL*|*_AUTH*) unset "$v";; esac; done
export HOME=$R/userhome CODEX_HOME=$R/home CORBANU_HOME=$R/home PFTERMINAL_HOME=$R/home CORBANU_TEST_NO_NATIVE_KEYRING=1 TERM=xterm-256color
export CLAUDE_CONFIG_DIR=$R/claude-config PATH="${CLAUDE_BIN_DIR:+$CLAUDE_BIN_DIR:}$PATH"
ZAI_API_KEY="$(env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME HOME=$REAL_HOME $REAL_HOME/.local/bin/corbanu vault auth-helper provider/zai_api_key)" \
CLAUDE_CODE_OAUTH_TOKEN="$(env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME HOME=$REAL_HOME $REAL_HOME/.local/bin/corbanu vault auth-helper claude-plan-test-token)" exec $BIN -C $R/ws -c log_dir=\"$R/logs\" -m glm-5.3-flash -c model_provider=\"zai\"
