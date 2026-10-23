# Source after env.sh and tmuxlib.sh. #555 homes (code-blind design, original-proposal.md):
#   z1 = H-ZAI: zai configured, default (ZAI_API_KEY env in the TUI only), work, personal; [provider_accounts] zai = "personal"
#   o1 = H-OAI: openai configured with a usable (fake, mock) default API key; zai `work` enrolled
#   o2 = H-OAI-bare: like o1, no zai account
#   n1 = H-NOKEY: zai configured, no default key, work enrolled (conversations exist)
# Real Z.AI key reaches enrolment via stdin only; all named zai accounts hold the same real key.
ZP=34741; ZU=http://127.0.0.1:$ZP
zcfg() { # zcfg <home> <provider> [provider_accounts-line]
  mkdir -p $1
  { if [ $2 = zai ]; then echo 'model = "glm-5.3-flash"'; else echo 'model = "gpt-5.4"'; fi
    echo "model_provider = \"$2\""
    echo 'approval_policy = "never"'; echo 'sandbox_mode = "danger-full-access"'
    echo "openai_base_url = \"$ZU/v1\""; echo "chatgpt_base_url = \"$ZU/backend-api/codex\""
    echo 'suppress_unstable_features_warning = true'
    echo '[features]'; echo 'named_accounts = true'
    [ -n "${3:-}" ] && { echo '[provider_accounts]'; echo "$3"; }
    echo "[projects.\"$S/ws\"]"; echo 'trust_level = "trusted"'; } > $1/config.toml
}
zenrol() { local h=$1; shift; for a in "$@"; do key provider/zai_api_key run_secret_stdin $CAND $h account add zai $a 2>&1 | sed "s#$S#\$S#g"; done; }
if [ "${1:-}" = init ]; then
  export FH=$S/fh-z; mkdir -p $FH
  for h in z1 o1 o2 n1; do [ -d $S/homes/$h ] && rm -r $S/homes/$h < /dev/null; done
  zcfg $S/homes/z1 zai 'zai = "personal"'; zenrol $S/homes/z1 work personal
  zcfg $S/homes/o1 openai; zenrol $S/homes/o1 work
  zcfg $S/homes/o2 openai
  for h in o1 o2; do printf 'sk-canary-defaultkey-S6B' | run $CAND $S/homes/$h login --with-api-key 2>&1 | tail -1; done
  zcfg $S/homes/n1 zai; zenrol $S/homes/n1 work
  for h in z1 o1 o2 n1; do echo "--- $h"; run $CAND $S/homes/$h account list 2>&1; done
fi
export FH=$S/fh-z
