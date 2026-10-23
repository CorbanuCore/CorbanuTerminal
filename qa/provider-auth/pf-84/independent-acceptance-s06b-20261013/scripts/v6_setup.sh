# Source after tdrv.sh. Home t2: zai-configured, default OpenAI login = ChatGPT (mock user tuidefault) in auth.json,
# named ChatGPT `work` (mock user tuiwork2). No OPENAI_API_KEY anywhere. Mock: the t1 mock (34731).
export FH=$S/fh-t2 H=$S/homes/t2
XENV=(CODEX_REFRESH_TOKEN_URL_OVERRIDE=$U/oauth/token CORBANU_TEST_LOGIN_ISSUER=$U)
if [ "${1:-}" = init ]; then
  for d in $H $FH; do [ -d $d ] && rm -r $d < /dev/null; done; mkdir -p $H $FH
  printf 'model = "glm-5.3-flash"\nmodel_provider = "zai"\napproval_policy = "never"\nsandbox_mode = "danger-full-access"\nopenai_base_url = "%s/v1"\nchatgpt_base_url = "%s/backend-api/codex"\nsuppress_unstable_features_warning = true\n[features]\nnamed_accounts = true\n[projects."%s/ws"]\ntrust_level = "trusted"\n' $U $U $S > $H/config.toml
  curl -s -X POST -H 'content-type: application/json' -d '{"next_user":"tuidefault","ttl":3600,"refresh_ttl":3600}' $U/_ctl
  show run $CAND $H login --device-auth --experimental_issuer $U < /dev/null | tail -2
  show run $CAND $H login --device-auth --experimental_issuer $U -c 'model_provider="openai"' -c 'model="gpt-6-luna"' < /dev/null | tail -2
  curl -s -X POST -H 'content-type: application/json' -d '{"next_user":"tuiwork2"}' $U/_ctl
  show run $CAND $H login --account work --device-auth --experimental_issuer $U < /dev/null | tail -2
  echo "auth.json: $(shasum -a 256 $H/auth.json | cut -c1-16)"
fi
