S=/Volumes/CorbanuDrive/Corbanu/tmp/pf84acc-scratch; B=$S/bin/corbanu-main
for hl in "s02/hG provider/zai/accounts/main/api_key" "s02/hG provider/zai_api_key" "s02/hC provider/claude-plan/accounts/real/claude_oauth_token" "s02/hP provider/claude-plan/accounts/cfgacct/claude_config_dir"; do
  set -- $hl
  env -i HOME=$S/fakehome PATH=/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$S/$1 $B vault auth-helper $2 > $S/out.bin 2>$S/e.txt; rc=$?
  echo "vault auth-helper $2 (home $1): exit=$rc stdout_bytes=$(wc -c < $S/out.bin | tr -d ' ') stderr=$(head -c 250 $S/e.txt | tr '\n' ' ')"
done
rm $S/out.bin < /dev/null
