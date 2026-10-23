. /Volumes/CorbanuDrive/Corbanu/tmp/pf84acc-scratch/realenv.sh; O=$S/s02/hO
run() { label="$1"; shift; out=$(rx $O "$@" 2>&1); rc=$?; echo "== $label (exit $rc)"; echo "   args: $*"; printf '%s\n' "$out" | summ; }
E=(); run "O1 openai default (API-key login), flag on"
E=(); run "O2 openai named work (descoped to S04: must fail closed)" -c provider_accounts.openai='"work"'
E=(); run "O3 amazon-bedrock named work (descoped: must fail closed)" -c provider_accounts.amazon-bedrock='"work"'
E=(); run "O4 flag OFF + openai selector work: ignored, default works" --disable named_accounts -c provider_accounts.openai='"work"'
