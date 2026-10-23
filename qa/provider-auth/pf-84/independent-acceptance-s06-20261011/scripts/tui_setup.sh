# Source after env.sh, tmuxlib.sh, aws_setup.sh. Home t1 for the TUI run: GLM 5.3 Flash on Z.AI (real default
# key in the TUI's env only), Bedrock and OpenAI pointed at the local mock (port 34631, started separately:
# python3 mock.py --port 34631 --log $S/logs/mock-t.jsonl --bearer fakekey=sk-fake-tui-S06CANARY
#   --bearer envbearer=fakebearer-env-S06CANARY --bearer envkey=sk-envdecoy-S06CANARY).
P=34631; U=http://127.0.0.1:$P
export FH=$S/fh-t1 H=$S/homes/t1
XENV=(CODEX_REFRESH_TOKEN_URL_OVERRIDE=$U/oauth/token CORBANU_TEST_LOGIN_ISSUER=$U)
DECOYS=(AWS_ACCESS_KEY_ID=AKIDENVDECOY AWS_SECRET_ACCESS_KEY=fakesecret-env-S06CANARY AWS_PROFILE=other
  AWS_BEARER_TOKEN_BEDROCK=fakebearer-env-S06CANARY OPENAI_API_KEY=sk-envdecoy-S06CANARY)
if [ "${1:-}" = init ]; then
[ -d $S/homes/t1 ] && rm -r $S/homes/t1 $S/fh-t1 < /dev/null
aws_setup t1 $P yes
cat > $H/config.toml <<EOF
model = "glm-5.3-flash"
model_provider = "zai"
approval_policy = "never"
sandbox_mode = "danger-full-access"
openai_base_url = "$U/v1"
chatgpt_base_url = "$U/backend-api/codex"
suppress_unstable_features_warning = true
[features]
named_accounts = true
[model_providers.amazon-bedrock]
base_url = "$U/openai/v1"
[model_providers.amazon-bedrock.aws]
region = "us-west-2"
[projects."$S/ws"]
trust_level = "trusted"
EOF
run $CAND $H account add amazon-bedrock work --kind aws-profile --value work
run $CAND $H account add amazon-bedrock gone --kind aws-profile --value nosuchprofile
curl -s -X POST -H 'content-type: application/json' -d '{"next_user":"tuiwork","ttl":3600,"refresh_ttl":3600}' $U/_ctl
echo "(with model_provider = zai the CLI refuses a ChatGPT sign-in; sign in with -c model_provider=openai -c model=gpt-5.4)"
run $CAND $H login --account work --device-auth --experimental_issuer $U < /dev/null | tail -1
run $CAND $H login --account work --device-auth --experimental_issuer $U -c 'model_provider="openai"' -c 'model="gpt-5.4"' < /dev/null | tail -2
printf 'sk-fake-tui-S06CANARY' | run $CAND $H login --account fake --with-api-key
run $CAND $H account list
# Worker script the GLM coordinator is asked to run (exec workers on other providers' named accounts).
cat > $S/ws/workers.sh <<'EOF'
#!/bin/bash
w() { echo "--- worker: $*"; corbanu exec --skip-git-repo-check "$@" "Reply with exactly: pong" < /dev/null 2>&1 \
  | grep -E '^mock:|ERROR: (account|OpenAI|Fatal)|^Error' | sort -u | cut -c1-220; echo "exit=${PIPESTATUS[0]}"; }
w -c 'model_provider="amazon-bedrock"' -m openai.gpt-5.4 --account work
w -c 'model_provider="amazon-bedrock"' -m openai.gpt-5.4 --account gone
w -c 'model_provider="openai"' -m gpt-5.4 --account fake
w -c 'model_provider="openai"' -m gpt-5.4 --account work
EOF
chmod +x $S/ws/workers.sh
fi
# Decoys are inherited by the TUI and by every worker it starts (fake values).
XENV+=("${DECOYS[@]}")
