# Home hP on provider `zpx` (custom chat provider -> local canary proxy -> real Z.AI glm-5.3-flash).
# Each account holds a synthetic canary; the proxy forwards default/main/work with the real key and
# answers 401 for every other canary. Run as: bash proxy-setup.sh
. "$(dirname "$0")/../env.sh"
H=$S/homes/hP; mkdir -p $H $S/proxy
python3 - $S/proxy/canaries.json <<'PY'
import json, secrets, sys
ids = ['default', 'main', 'work', 'fake', 'kimionly', 'aggr']
json.dump({'cnry-%s-%s' % (i, secrets.token_hex(10)): i for i in ids}, open(sys.argv[1], 'w'))
PY
can() { python3 -c "import json;d=json.load(open('$S/proxy/canaries.json'));print(next(k for k,v in d.items() if v=='$1'))"; }
cat > $H/config.toml <<EOF
model = "glm-5.3-flash"
model_provider = "zpx"
suppress_unstable_features_warning = true

[features]
named_accounts = true

[model_providers.zpx]
name = "Z.AI via canary proxy"
base_url = "http://127.0.0.1:18585/proxy"
env_key = "ZPX_API_KEY"
wire_api = "chat"
request_max_retries = 0
stream_max_retries = 0

[projects."$S/work"]
trust_level = "trusted"
EOF
for a in main work fake aggr; do addval $H "$(can $a)" zpx $a >/dev/null; done
addval $H "$(can kimionly)" kimi-code kimionly >/dev/null
echo "home hP accounts:"; cx $H account list | sed 's/^/  /'
echo "default account = env ZPX_API_KEY (canary 'default'); proxy forwards: default,main,work; 401 for fake,kimionly,aggr"
tmux -L pf84s03acc kill-session -t proxy 2>/dev/null
# the key is fetched inside the proxy's own command (never on a command line, never printed)
tmux -L pf84s03acc new-session -d -s proxy "bash $S/scripts/s03/proxy-run.sh"
sleep 2; tmux -L pf84s03acc capture-pane -p -t proxy | grep -v '^$' | head -3; echo "proxy session started"
