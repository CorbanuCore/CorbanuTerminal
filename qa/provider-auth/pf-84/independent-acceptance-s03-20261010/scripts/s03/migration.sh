# Existing single-account homes made by the PRE-PF-84 build carry over untouched with the S03 candidate.
# hM: Z.AI default key saved by pre-PF-84 onboarding (masked paste), sessions from the pre build.
# hO: OpenAI API-key login (gpt-5.4) by the pre-PF-84 build. Run as: bash migration.sh
. "$(dirname "$0")/../env.sh"
M=$S/homes/hM; O=$S/homes/hO
hs() { local h=$1; ( cd $h; for f in config.toml secrets/local.age auth.json $(find secrets/keyring-fallback -type f 2>/dev/null | sort); do [ -f $f ] && printf '%s=%s ' "$f" "$(shasum -a 256 < $f | cut -c1-12)"; done ); echo; }
px() { local h=$1; shift; (cd $S/work && env -i HOME=$S/fakehome PATH=/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$h $PRE "$@"); }
r() { local label=$1 h=$2; shift 2; local out rc; out=$(cx $h "$@" 2>&1); rc=$?
  echo "== $label (exit $rc)"; echo "   args: $*"; printf '%s\n' "$out" | grep -E '^pong|^ERROR|^Error|warning' | sort -u | head -3 | sed 's/^/   | /'; echo "   hashes: $(hs $h)"; }
TUI_SID=$(cat $S/hM-tui-session)
out=$(px $M exec --skip-git-repo-check "$PROMPT" 2>&1); EXEC_SID=$(printf '%s\n' "$out" | grep -E '^session id:' | sed 's/^session id: //')
echo "pre-PF-84 exec on hM: $(printf '%s\n' "$out" | grep -c '^pong') pong; session $EXEC_SID; pre TUI session $TUI_SID"
echo "hM baseline (after pre-PF-84 onboarding + sessions): $(hs $M)"
r "M1 candidate exec, flag off (default)" $M exec --skip-git-repo-check "$PROMPT"
r "M2 candidate exec --enable named_accounts" $M exec --skip-git-repo-check --enable named_accounts "$PROMPT"
r "M3 candidate exec --enable named_accounts --account default" $M exec --skip-git-repo-check --enable named_accounts --account default "$PROMPT"
r "M4 candidate exec --enable named_accounts --account main (nothing enrolled)" $M exec --skip-git-repo-check --enable named_accounts --account main "$PROMPT"
r "M5 candidate account list --enable named_accounts" $M account list --enable named_accounts
r "M6 candidate exec resume <pre-PF-84 exec session> --enable named_accounts" $M exec --skip-git-repo-check --enable named_accounts resume $EXEC_SID "$PROMPT"
r "M7 candidate exec resume <pre-PF-84 TUI session> --enable named_accounts -c provider_accounts.zai=\"main\"" $M exec --skip-git-repo-check --enable named_accounts -c 'provider_accounts.zai="main"' resume $TUI_SID "$PROMPT"
r "M8 candidate exec resume <pre-PF-84 TUI session>, flag off" $M exec --skip-git-repo-check resume $TUI_SID "$PROMPT"
out=$(px $M exec --skip-git-repo-check resume $EXEC_SID "$PROMPT" 2>&1); echo "== M9 pre-PF-84 build resumes its exec session after the candidate runs: $(printf '%s\n' "$out" | grep -c '^pong') pong"
echo; echo "### OpenAI gpt-5.4 (API-key login made by the pre-PF-84 build)"
mkdir -p $O; printf 'model = "gpt-5.4"\nmodel_provider = "openai"\n' > $O/config.toml
X="$(helper openai-api-key)" bash -c 'printf %s "$X" | env -i HOME='$S'/fakehome PATH=/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME='$O' '$PRE' login --with-api-key' 2>&1 | sed 's/^/   pre login: /'
out=$(px $O exec --skip-git-repo-check "$PROMPT" 2>&1); echo "pre-PF-84 exec on hO: $(printf '%s\n' "$out" | grep -c '^pong') pong"
echo "hO baseline: $(hs $O)"
r "O1 candidate exec, flag off" $O exec --skip-git-repo-check "$PROMPT"
r "O2 candidate exec --enable named_accounts --account default" $O exec --skip-git-repo-check --enable named_accounts --account default "$PROMPT"
r "O3 candidate exec --enable named_accounts --account work (named OpenAI sign-ins are descoped to S04)" $O exec --skip-git-repo-check --enable named_accounts --account work "$PROMPT"
r "O4 candidate account add openai work --enable named_accounts (stdin: synthetic)" $O account add openai work --enable named_accounts < <(printf 'sk-fake-%s' "$(openssl rand -hex 8)")
