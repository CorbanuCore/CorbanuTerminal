# Second provider: Kimi k3 on kimi-code. exec --account main|fake|ghost, resume, cross-provider prefix.
# Run as: X="$(helper provider/zai_api_key)" bash exec-kimi.sh  (X = real Z.AI default key for the cross-provider rows)
. "$(dirname "$0")/../env.sh"
H=$S/homes/hK; mkdir -p $H
cat > $H/config.toml <<'EOF'
model = "kimi-k3"
model_provider = "kimi-code"
suppress_unstable_features_warning = true

[features]
named_accounts = true
EOF
addreal provider/kimi_api_key $H kimi-code main >/dev/null 2>&1
addval $H "fake-kimi-$(openssl rand -hex 12)" kimi-code fake >/dev/null 2>&1
addreal provider/zai_api_key $H zai zmain >/dev/null 2>&1
echo "home hK accounts (no default Kimi key anywhere):"; cx $H account list | sed 's/^/  /'
E=(ZAI_API_KEY="$X")
sid() { printf '%s\n' "$1" | grep -E '^session id:' | head -1 | sed 's/^session id: //'; }
run() { local label=$1; shift; local out rc
  out=$(cx $H exec --skip-git-repo-check "$@" "$PROMPT" 2>&1); rc=$?; LAST=$(sid "$out")
  echo "== $label (exit $rc)"; echo "   args: exec $*"; printf '%s\n' "$out" | summ; }
run "K1 --account main" --account main; K1=$LAST
run "K2 --account fake" --account fake; K2=$LAST
run "K3 --account ghost" --account ghost
run "K4 default account (no Kimi default key configured)"
run "K5 resume K2 (recorded fake), no --account" resume $K2
run "K6 resume K2 with --account main" resume --account main $K2
run "K7 resume K1 (recorded main), no --account" resume $K1
echo "-- cross-provider prefix: session provider is kimi-code"
run "K8 --account zai:zmain (account of another provider; session stays kimi-code)" --account zai:zmain
run "K9 --account zai:ghost (missing account of another provider)" --account zai:ghost
run "K10 -c model_provider=zai -m glm-5.3-flash --account zai:zmain" -c model_provider='"zai"' -m glm-5.3-flash --account zai:zmain
run "K11 -c model_provider=zai -m glm-5.3-flash --account kimi-code:main (prefix names the other provider)" -c model_provider='"zai"' -m glm-5.3-flash --account kimi-code:main
run "K12 -c model_provider=zai -m glm-5.3-flash --account kimi-code:ghost (missing, other provider)" -c model_provider='"zai"' -m glm-5.3-flash --account kimi-code:ghost
