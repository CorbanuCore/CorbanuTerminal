. /Volumes/CorbanuDrive/Corbanu/tmp/pf84acc-scratch/realenv.sh; K=$S/s02/hK
run() { label="$1"; shift; out=$(rx $K "$@" 2>&1); rc=$?; echo "== $label (exit $rc)"; echo "   args: $*"; printf '%s\n' "$out" | summ; }
E=(); run "K1 kimi-code named main (real key)" -c provider_accounts.kimi-code='"main"'
E=(); run "K2 kimi-code named fake" -c provider_accounts.kimi-code='"fake"'
E=(); run "K3 kimi-code named ghost" -c provider_accounts.kimi-code='"ghost"'
