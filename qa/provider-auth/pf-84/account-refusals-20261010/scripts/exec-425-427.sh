# #425 / #427-2 exec matrix on real Z.AI GLM 5.3 Flash and Kimi. Usage: B=<bin> bash exec-425-427.sh <tag>
# Proof of which account: the only Z.AI credential in hZ besides the synthetic `fake` is the DEFAULT key
# (env ZAI_API_KEY), so a `pong` on a zai session means the default account answered. In hK the only Kimi
# credential is the default KIMI_API_KEY.
. "$(dirname "$0")/live-env.sh"
tag=$1; H=$S/homes/hZ-$tag; HK=$S/homes/hK-$tag; mkdir -p $H $HK
for h in $H; do cat > $h/config.toml <<'C'
model = "glm-5.3-flash"
model_provider = "zai"
suppress_unstable_features_warning = true

[features]
named_accounts = true
C
done
cat > $HK/config.toml <<'C'
model = "kimi-k3"
model_provider = "kimi-code"
suppress_unstable_features_warning = true

[features]
named_accounts = true
C
addreal provider/zai_api_key $H zai main >/dev/null 2>&1
addval $H "fake-zai-$(openssl rand -hex 12)" zai fake >/dev/null 2>&1
echo "binary: $tag"; echo "hZ accounts:"; cx $H account list | sed 's/^/  /'
sid() { printf '%s\n' "$1" | grep -E '^session id:' | head -1 | sed 's/^session id: //'; }
run() { local home=$1 label=$2; shift 2; local out rc
  out=$(cx $home exec --skip-git-repo-check "$@" "$PROMPT" 2>&1); rc=$?; LAST=$(sid "$out")
  echo "== $label (exit $rc)"; echo "   args: exec $*"; printf '%s\n' "$out" | summ; }
export ZAI_API_KEY="$X"; KEEP=(ZAI_API_KEY)
echo "-- #425: zai session, account of another provider"
run $H "X12 --account kimi-code:main (kimi-code has no accounts)" --account kimi-code:main
run $H "X12b --account kimi-code:ghost" --account kimi-code:ghost
run $H "Z3 --account ghost (session provider, reference)" --account ghost
run $H "Z1 --account main" --account main; Z1=$LAST
echo "-- #427-2: flag-off resume of a thread recorded on main"
run $H "R7 resume --disable named_accounts <main session>" resume --disable named_accounts $Z1
run $H "R7b resume --disable named_accounts --account default <main session>" resume --disable named_accounts --account default $Z1
echo "-- #425 positive / applies-to-nothing: kimi-code:kmain configured (real Kimi key)"
addreal provider/kimi_api_key $H kimi-code kmain >/dev/null 2>&1
run $H "P1 --account kimi-code:kmain (configured; session stays on zai default)" --account kimi-code:kmain
run $H "P2 --account kimi-code:kmain, agents.provider_allowlist=[zai]" -c 'agents.provider_allowlist=["zai"]' --account kimi-code:kmain
run $H "P3 --account kimi-code:kmain, --disable multi_agent" --disable multi_agent --account kimi-code:kmain
echo "-- #425: kimi-code session"
unset ZAI_API_KEY; export KIMI_API_KEY="$K"; KEEP=(KIMI_API_KEY)
run $HK "K9 --account zai:ghost (missing account of another provider)" --account zai:ghost
run $HK "K0 no --account (reference: Kimi default key)"
