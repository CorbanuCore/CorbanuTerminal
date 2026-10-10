. /Volumes/CorbanuDrive/Corbanu/tmp/pf84acc-scratch/realenv.sh; C=$S/s02/hC
rxs() { home=$1; shift; (cd $S/fakehome && env -i HOME=$S/fakehome PATH=$S/shim:$S/pathbin:/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$home $B exec --skip-git-repo-check "$@" "Reply with exactly the word pong and nothing else."); }
run() { label="$1"; shift; out=$(rxs $C "$@" 2>&1); rc=$?; echo "== $label (exit $rc)"; echo "   args: $*"; printf '%s\n' "$out" | summ; }
run "P1 claude-plan named real (claude-plan-test-token)" -c provider_accounts.claude-plan='"real"'
run "P2 claude-plan named fake" -c provider_accounts.claude-plan='"fake"'
: > $S/pathbin/calls.log; run "P3 claude-plan named ghost" -c provider_accounts.claude-plan='"ghost"'
echo "   auth-command calls (P3 only): $(wc -l < $S/pathbin/calls.log | tr -d ' ')"; sed 's/^[0-9]* /   | /' $S/pathbin/calls.log | sort | uniq -c
