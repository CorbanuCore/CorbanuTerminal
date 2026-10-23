. /Volumes/CorbanuDrive/Corbanu/tmp/pf84acc-scratch/realenv.sh; G=$S/s02/hG
run() { label="$1"; shift; out=$(rx $G "$@" 2>&1); rc=$?; echo "== $label (exit $rc)"; printf '%s\n' "$out" | summ; }
echo "home G config.toml (selector in the config table, no -c):"; sed 's/^/   | /' $G/config.toml
E=(); run "T1 [provider_accounts] zai = \"fake\" in config.toml"
E=(); run "T2 same config, -c provider_accounts.zai=\"main\" overrides it" -c provider_accounts.zai='"main"'
