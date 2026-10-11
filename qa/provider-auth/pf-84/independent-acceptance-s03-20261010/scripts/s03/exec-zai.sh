# exec --account / precedence / flag-off / resume matrix on real Z.AI (glm-5.3-flash).
# Run as: X="$(helper provider/zai_api_key)" bash exec-zai.sh   (X = real default key, never printed)
. "$(dirname "$0")/../env.sh"
H=$S/homes/hZ; mkdir -p $H
cat > $H/config.toml <<'EOF'
model = "glm-5.3-flash"
model_provider = "zai"
suppress_unstable_features_warning = true

[features]
named_accounts = true
EOF
addreal provider/zai_api_key $H zai main >/dev/null 2>&1
addval $H "fake-zai-$(openssl rand -hex 12)" zai fake >/dev/null 2>&1
addval $H "fake-zai-$(openssl rand -hex 12)" zai spare >/dev/null 2>&1
echo "home hZ accounts (default key only in env ZAI_API_KEY):"; cx $H account list | sed 's/^/  /'
E=(ZAI_API_KEY="$X")
run() { local label=$1; shift; local out rc
  out=$(cx $H exec --skip-git-repo-check "$@" "$PROMPT" 2>&1); rc=$?
  echo "== $label (exit $rc)"; echo "   args: $*"; printf '%s\n' "$out" | summ
  printf '%s\n' "$out" | grep -E '^session id:' | head -1 | sed 's/^session id: /   session=/' > $S/last-session
  cat $S/last-session; }
run "X1 --account main (real key; valid ZAI_API_KEY also in env)" --account main
run "X2 --account fake (env holds a valid key: must not be used)" --account fake
run "X3 --account ghost (not configured)" --account ghost
run "X4 --account zai:main (qualified)" --account zai:main
run "X5 --account zai:fake (qualified)" --account zai:fake
run "X6 --account default" --account default
run "X7 precedence: config fake, --account main" -c 'provider_accounts.zai="fake"' --account main
run "X8 precedence: config main, --account fake" -c 'provider_accounts.zai="main"' --account fake
run "X9 config fake, --account default" -c 'provider_accounts.zai="fake"' --account default
run "X10 config only: provider_accounts.zai=fake (no --account)" -c 'provider_accounts.zai="fake"'
run "X11 --account Bad_Name (invalid)" --account Bad_Name
run "X12 --account kimi-code:main (another provider; not configured there)" --account kimi-code:main
run "X13 --account nosuchprov:main" --account nosuchprov:main
run "X14 flag OFF: --disable named_accounts --account fake" --disable named_accounts --account fake
run "X15 flag OFF: --disable named_accounts --account default" --disable named_accounts --account default
run "X16 flag OFF: --disable named_accounts, no --account" --disable named_accounts
run "X17 flag OFF: --disable named_accounts --account main" --disable named_accounts --account main
