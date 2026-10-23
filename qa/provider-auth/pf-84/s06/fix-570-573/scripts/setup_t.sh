# Source after env.sh: TUI home on Z.AI (fake key, mock), OpenAI + Bedrock at the mock.
# usage: setup_t <home-name> <port>
setup_t() {
  local n=$1; export P=$2 U=http://127.0.0.1:$2
  export H=$S/homes/$n FH=$S/fh-$n
  [ -d $H ] && rm -r $H < /dev/null; [ -d $FH ] && rm -r $FH < /dev/null
  source $Q/aws_setup.sh; aws_setup $n $P yes
  cat > $H/config.toml <<EOC
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
EOC
  XENV=(CODEX_REFRESH_TOKEN_URL_OVERRIDE=$U/oauth/token CORBANU_TEST_LOGIN_ISSUER=$U)
  run ${SETUPBIN:-$BASE} $H account add amazon-bedrock work --kind aws-profile --value work
  ctl '{"next_user":"tuiwork","ttl":3600,"refresh_ttl":3600}'
  # Signed in with the OpenAI provider forced so the BASE-equivalent sign-in works on every build.
  run ${SETUPBIN:-$BASE} $H login --account work --device-auth --experimental_issuer $U -c 'model_provider="openai"' -c 'model="gpt-5.4"' < /dev/null 2>&1 | tail -1
  printf 'sk-fake-tui-S06CANARY' | run ${SETUPBIN:-$BASE} $H login --account fake --with-api-key 2>&1 | tail -1
  run ${SETUPBIN:-$BASE} $H account list
  XENV+=(ZAI_API_KEY=fake-zai-S06CANARY OPENAI_API_KEY=sk-envdecoy-S06CANARY)
}
