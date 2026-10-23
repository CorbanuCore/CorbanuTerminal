#!/bin/bash
# usage: launch.sh <name> <flag:on|off> [extra corbanu args]
# Disposable home + workspace, GLM 5.2 on zai, synthetic GHE credential and canary.
E=$HOME/corbanu-rtx/pf27s02-e2e
BIN=$HOME/corbanu-rtx/target-pf27s02/debug/corbanu
NAME=$1; FLAG=$2; shift 2
H=$E/homes/$NAME; W=$E/work/$NAME
mkdir -p "$H/secrets" "$H/shell_snapshots" "$W" "$E/logs/$NAME"
echo '{"OPENAI_API_KEY":"sk-pf27-synthetic-auth-json-0000000000"}' > "$H/auth.json"
echo "synthetic vault blob" > "$H/secrets/local.age"
echo "PF27_DOTENV=synthetic" > "$H/.env"
echo "export SYNTHETIC=1" > "$H/shell_snapshots/s.sh"
[ -d "$W/.git" ] || (cd "$W" && git init -q && echo "# demo" > README.md && git add . && git -c user.email=t@t -c user.name=t commit -qm init)
cat > "$W/check.sh" <<"S"
printf "credential-looking names: %s\n" "$(env | cut -d= -f1 | grep -iE "key|token|secret|pass|auth" | sort | tr "\n" " ")"
printf "agent token sha256: %s\n" "$(printf %s "$GH_ENTERPRISE_TOKEN" | sha256sum | cut -c1-12)"
curl -sS https://127.0.0.1.nip.io:8443/api/v3/user -H "Authorization: Bearer $GH_ENTERPRISE_TOKEN"
echo
S
cat > "$W/probe.sh" <<"S"
probe() { if eval "$2" >/dev/null 2>&1; then echo "$1: ALLOWED"; else echo "$1: denied"; fi; }
probe "read auth.json" "cat \"\$CODEX_HOME/auth.json\""
probe "read secrets/" "cat \"\$CODEX_HOME\"/secrets/*"
probe "read .env" "cat \"\$CODEX_HOME/.env\""
probe "read config.toml" "cat \"\$CODEX_HOME/config.toml\""
probe "read shell snapshots" "ls \"\$CODEX_HOME/shell_snapshots/\""
probe "write config.toml" "printf \"\n\" >> \"\$CODEX_HOME/config.toml\""
probe "read Core environment" "grep -qa ZAI_API_KEY= /proc/\$(cat \"\$PWD/core.pid\")/environ"
echo probes done
S
FEAT=""; [ "$FLAG" = on ] && FEAT="secretless_agent_launch = true"
cat > "$H/config.toml" <<CFG
model = "glm-5.2"
model_provider = "zai"
approval_policy = "never"
sandbox_mode = "workspace-write"
suppress_unstable_features_warning = true
[sandbox_workspace_write]
network_access = true
[features]
isolated_credential_broker = true
$FEAT
[features.network_proxy]
enabled = true
allow_local_binding = true
[features.network_proxy.domains]
"127.0.0.1.nip.io" = "allow"
[projects."$W"]
trust_level = "trusted"
CFG
cd "$W"
echo $$ > "$W/core.pid"
export CORBANU_HOME="$H" CODEX_HOME="$H"
unset PFTERMINAL_HOME
export GH_HOST=127.0.0.1.nip.io SSL_CERT_FILE=$E/fixture/ca.pem
GH_ENTERPRISE_TOKEN="$(cat $E/fixture/synthetic-token)" PF27_CANARY_API_KEY="$(cat $E/canary-api-key)" ZAI_API_KEY="$(cat $E/zai.key)" RUST_LOG=info exec "$BIN" -m glm-5.2 -c model_provider="\"zai\"" -c log_dir="\"$E/logs/$NAME\"" "$@"
