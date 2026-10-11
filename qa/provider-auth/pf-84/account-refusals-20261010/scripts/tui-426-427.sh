# Usage: B=<bin> bash tui-426-427.sh <tag>
. "$(dirname "$0")/tui-lib.sh"
tag=$1; H=$S/homes/hT-$tag; mkdir -p $H
cat > $H/config.toml <<'C'
model = "glm-5.3-flash"
model_provider = "zai"
suppress_unstable_features_warning = true

[features]
named_accounts = true

[projects."/Volumes/CorbanuDrive/Corbanu/tmp/pf84fix2/work"]
trust_level = "trusted"
C
addval $H "fake-zai-$(openssl rand -hex 12)" zai fake >/dev/null 2>&1
echo "binary: $tag"; cx $H account list | sed 's/^/  /'
# A session recorded on `fake` (exec; 401) and one on default.
export ZAI_API_KEY="$X"; KEEP=(ZAI_API_KEY)
FAKE=$(cx $H exec --skip-git-repo-check --account fake "$PROMPT" 2>&1 | grep -E '^session id:' | head -1 | sed 's/^session id: //')
DEF=$(cx $H exec --skip-git-repo-check "$PROMPT" 2>&1 | grep -E '^session id:' | head -1 | sed 's/^session id: //')
echo "recorded: fake=$FAKE default=$DEF"
T kill-server 2>/dev/null
echo "== T1 #426: corbanu --account ghost"; start t1 $H --account ghost; sleep 6; snap t1
echo "== T2 #426: corbanu resume --account ghost <default session>"; start t2 $H resume --account ghost $DEF; sleep 6; snap t2
echo "== T3 #425: corbanu --account kimi-code:ghost"; start t3 $H --account kimi-code:ghost; sleep 6; snap t3
echo "== T4 #427: corbanu resume <session recorded on fake>, send ping"; start t4 $H resume $FAKE; sleep 8; snap t4 >/dev/null
send t4 "ping"; sleep 10; snap t4 | grep -E -i 'rejected|account|providers|recover|401|unauthor' | head -6
send t4 "/exit"; sleep 2
T kill-server 2>/dev/null
