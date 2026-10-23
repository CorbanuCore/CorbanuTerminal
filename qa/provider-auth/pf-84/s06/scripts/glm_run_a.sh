#!/usr/bin/env bash
# PF-84-S06 slice A GLM run: a GLM 5.3 Flash coordinator (real default Z.AI key from the
# installed vault helper, never printed) runs a `corbanu exec` worker on the named Bedrock
# account `fake` (a profile that does not exist). The worker is refused before any request;
# the coordinator still replies. Disposable homes, CORBANU_TEST_NO_NATIVE_KEYRING=1.
# usage: glm_run_a.sh <debug corbanu> <out dir>
set -u
BIN=$1; OUT=$2; mkdir -p "$OUT"; OUT=$(cd "$OUT" && pwd)
REAL_HOME=$HOME
RUN=$(mktemp -d); export HOME=$RUN/home CODEX_HOME=$RUN/corbanu
mkdir -p "$HOME" "$CODEX_HOME" "$RUN/ws"
export CORBANU_TEST_NO_NATIVE_KEYRING=1
unset CORBANU_HOME PFTERMINAL_HOME AWS_PROFILE AWS_ACCESS_KEY_ID AWS_SECRET_ACCESS_KEY AWS_BEARER_TOKEN_BEDROCK
cat > "$CODEX_HOME/config.toml" <<EOF
suppress_unstable_features_warning = true
[features]
named_accounts = true
[model_providers.amazon-bedrock]
base_url = "http://127.0.0.1:9/openai/v1"
[model_providers.amazon-bedrock.aws]
region = "us-west-2"
EOF
"$BIN" account add amazon-bedrock fake --kind aws-profile --value nosuchprofile > "$OUT/setup.txt" 2>&1
PROMPT="Run this shell command exactly once and show its last 3 lines verbatim in a code block: \
CODEX_HOME=$CODEX_HOME CORBANU_TEST_NO_NATIVE_KEYRING=1 $BIN exec --skip-git-repo-check -C $RUN/ws -c model_provider=\"amazon-bedrock\" -m openai.gpt-5.4 --account fake 'Reply with exactly: pong' 2>&1 | tail -3 . \
Do not retry it or change the account. Then, on a new last line, reply with the single word pong yourself."
ZAI_API_KEY="$(env -u CODEX_HOME -u CORBANU_HOME -u CORBANU_TEST_NO_NATIVE_KEYRING HOME="$REAL_HOME" "$REAL_HOME/.local/bin/corbanu" vault auth-helper provider/zai_api_key)" \
  "$BIN" exec --skip-git-repo-check -C "$RUN/ws" -s danger-full-access -c approval_policy="never" \
  -m glm-5.3-flash -c model_provider="zai" "$PROMPT" > "$OUT/glm-run.txt" 2>&1
echo "exit $?" >> "$OUT/glm-run.txt"
# Leak scan: the real key never appears in the output, logs or rollouts.
ZAI_API_KEY="$(env -u CODEX_HOME -u CORBANU_HOME -u CORBANU_TEST_NO_NATIVE_KEYRING HOME="$REAL_HOME" "$REAL_HOME/.local/bin/corbanu" vault auth-helper provider/zai_api_key)" \
  python3 - "$OUT" "$CODEX_HOME" > "$OUT/leak-scan.txt" <<'PY'
import os, sys, pathlib
key = os.environ["ZAI_API_KEY"].strip()
if len(key) < 16:
    sys.exit("no key from the vault helper; scan not run")
hits = []
for root in sys.argv[1:]:
    for path in pathlib.Path(root).rglob("*"):
        if path.is_file():
            try:
                data = path.read_bytes()
            except OSError:
                continue
            if key and (key.encode() in data or key[:16].encode() in data):
                hits.append(str(path))
print("files with the real key or a 16-char prefix: %d" % len(hits))
PY
