# Source after env.sh, tmuxlib.sh, aws_setup.sh. Home t1 for the TUI runs: GLM 5.3 Flash on Z.AI (real default key in
# the TUI's env only), Bedrock and OpenAI pointed at the local mock (port 34731, started separately:
# python3 mock.py --port 34731 --log $S/logs/mock-t.jsonl --bearer fakekey=sk-fake-tui-S6B --bearer envbearer=fakebearer-env-S06CANARY
#   --bearer envkey=sk-envdecoy-S6B --bearer tuiadd=sk-tuiadd-S6B).
P=34731; U=http://127.0.0.1:$P
export FH=$S/fh-t1 H=$S/homes/t1
XENV=(CODEX_REFRESH_TOKEN_URL_OVERRIDE=$U/oauth/token CORBANU_TEST_LOGIN_ISSUER=$U)
DECOYS=(AWS_ACCESS_KEY_ID=AKIDENVDECOY AWS_SECRET_ACCESS_KEY=fakesecret-env-S06CANARY AWS_PROFILE=other
  AWS_BEARER_TOKEN_BEDROCK=fakebearer-env-S06CANARY OPENAI_API_KEY=sk-envdecoy-S6B)
if [ "${1:-}" = init ]; then
[ -d $H ] && rm -r $H < /dev/null; [ -d $FH ] && rm -r $FH < /dev/null
aws_setup t1 $P yes
cat > $H/config.toml <<EOT
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
EOT
echo "## enrolment in t1 (zai-configured home)"
show run $CAND $H account add amazon-bedrock work --kind aws-profile --value work
show run $CAND $H account add amazon-bedrock gone --kind aws-profile --value nosuchprofile
curl -s -X POST -H 'content-type: application/json' -d '{"next_user":"tuiwork","ttl":3600,"refresh_ttl":3600}' $U/_ctl
echo "(#573: ChatGPT sign-in of a named account while model_provider = zai)"
show run $CAND $H login --account work --device-auth --experimental_issuer $U < /dev/null | grep -vE 'Follow these|Open this|one-time|MOCK-0001|Continue only|^\s*$|codex/device'
printf 'sk-fake-tui-S6B' | show run $CAND $H login --account fake --with-api-key
show run $CAND $H account list
ls $H
cat > $S/ws/workers.sh <<'EOT'
#!/bin/bash
w() { echo "--- worker: $*"; corbanu exec --skip-git-repo-check "$@" "Reply with exactly: pong" < /dev/null 2>&1 \
  | grep -E '^mock:|ERROR: |^Error' | sort -u | cut -c1-220; echo "exit=${PIPESTATUS[0]}"; }
w -c 'model_provider="amazon-bedrock"' -m openai.gpt-5.4 --account work
w -c 'model_provider="amazon-bedrock"' -m openai.gpt-5.4 --account gone
w -c 'model_provider="openai"' -m gpt-5.4 --account fake
w -c 'model_provider="openai"' -m gpt-5.4 --account work
EOT
chmod +x $S/ws/workers.sh
ln -sf $CAND $S/pathbin/corbanu
fi
XENV+=("${DECOYS[@]}")
