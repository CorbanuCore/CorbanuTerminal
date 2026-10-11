# Home hS on real Z.AI for in-process spawn: main and work hold the real key, fake/kimionly are synthetic,
# and the DEFAULT account is a synthetic invalid key (env ZAI_API_KEY), so anything that silently falls
# back to default gets a 401 while the selected real account answers.
# Run as: bash spawn-setup.sh
. "$(dirname "$0")/../env.sh"
H=$S/homes/hS; mkdir -p $H
cat > $H/config.toml <<EOF
model = "glm-5.3-flash"
model_provider = "zai"
suppress_unstable_features_warning = true

[features]
named_accounts = true

[projects."$S/work"]
trust_level = "trusted"
EOF
addreal provider/zai_api_key $H zai main >/dev/null
addreal provider/zai_api_key $H zai work >/dev/null
addval $H "fake-zai-$(openssl rand -hex 12)" zai fake >/dev/null
addval $H "fake-kimi-$(openssl rand -hex 12)" kimi-code kimionly >/dev/null
printf 'zai-default-canary-%s' "$(openssl rand -hex 10)" > $S/default-canary.txt
echo "home hS accounts:"; cx $H account list | sed 's/^/  /'
echo "default account: env ZAI_API_KEY = synthetic invalid canary (default => 401)"
