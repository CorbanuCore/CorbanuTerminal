# exec resume keeps the recorded account; explicit --account overrides; removed account blocks.
# Run as: X="$(helper provider/zai_api_key)" bash resume-zai.sh   (after exec-zai.sh, same home)
. "$(dirname "$0")/../env.sh"
H=$S/homes/hZ; E=(ZAI_API_KEY="$X")
sid() { printf '%s\n' "$1" | grep -E '^session id:' | head -1 | sed 's/^session id: //'; }
start() { local out; out=$(cx $H exec --skip-git-repo-check "$@" "$PROMPT" 2>&1); echo "$(sid "$out") $(printf '%s\n' "$out" | grep -c -E '^pong') $(printf '%s\n' "$out" | grep -c '401 Unauthorized')"; }
res() { local label=$1 id=$2; shift 2; local out rc
  out=$(cx $H exec --skip-git-repo-check resume "$@" "$id" "$PROMPT" 2>&1); rc=$?
  echo "== $label (exit $rc)"; echo "   args: exec resume $* <$id>"; printf '%s\n' "$out" | grep -v corbanu_call_metrics | summ; }
addreal provider/zai_api_key $H zai gone >/dev/null 2>&1
read SF pf uf <<<"$(start --account fake)";      echo "session F started with --account fake: pong=$pf 401=$uf id=$SF"
read SM pm um <<<"$(start --account main)";      echo "session M started with --account main: pong=$pm 401=$um id=$SM"
read SG pg ug <<<"$(start --account gone)";      echo "session G started with --account gone (real key): pong=$pg 401=$ug id=$SG"
read SC pc uc <<<"$(start -c 'provider_accounts.zai="fake"')"; echo "session C started with config provider_accounts.zai=fake: pong=$pc 401=$uc id=$SC"
read SD pd ud <<<"$(start --account default)";   echo "session D started with --account default: pong=$pd 401=$ud id=$SD"
read SO po uo <<<"$(start --disable named_accounts)"; echo "session O started with the flag off: pong=$po 401=$uo id=$SO"
res "R1 resume F (recorded fake), no --account" $SF
res "R2 resume F with --account main" $SF --account main
res "R3 resume M (recorded main), no --account, config says fake" $SM -c 'provider_accounts.zai="fake"'
res "R4 resume C (config-selected fake), config now empty" $SC
res "R5 resume D (explicit default), config says fake" $SD -c 'provider_accounts.zai="fake"'
res "R6 resume O (flag-off session) with flag on, config says fake" $SO -c 'provider_accounts.zai="fake"'
res "R7 resume F (recorded fake) with the flag OFF" $SF --disable named_accounts
res "R8 resume M with --account ghost" $SM --account ghost
echo "-- removing account gone"; cx $H account remove zai gone 2>&1 | sed 's/^/   | /'
res "R9 resume G (recorded gone, now removed), no --account" $SG
res "R10 resume G with --account main (explicit recovery)" $SG --account main
