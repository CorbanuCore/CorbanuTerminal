# Home hW: provider zcmd (chat -> canary proxy) whose credential comes from auth.command (authcmd.sh).
. "$(dirname "$0")/../env.sh"
H=$S/homes/hW; mkdir -p $H
cat > $H/config.toml <<EOT
model = "glm-5.3-flash"
model_provider = "zcmd"
suppress_unstable_features_warning = true

[features]
named_accounts = true

[model_providers.zcmd]
name = "Z.AI via canary proxy (auth.command)"
base_url = "http://127.0.0.1:18585/proxy"
wire_api = "chat"
request_max_retries = 0
stream_max_retries = 0

[model_providers.zcmd.auth]
command = "$S/scripts/s03/authcmd.sh"
timeout_ms = 5000

[projects."$S/work"]
trust_level = "trusted"
EOT
for a in work fake; do cx $H account add zcmd $a --kind command </dev/null 2>&1 | sed 's/^/  add: /'; done
echo "home hW accounts:"; cx $H account list | sed 's/^/  /'
