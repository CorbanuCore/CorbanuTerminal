S=/Volumes/CorbanuDrive/Corbanu/tmp/pf84acc-scratch; B=$S/bin/corbanu-main; G=$S/s02/hG
sh_() { shasum -a 256 "$@" | awk '{print substr($1,1,16)}'; }
snap() { echo "   local.age sha256[:16]=$(sh_ $G/secrets/local.age) size=$(stat -f %z $G/secrets/local.age)  config.toml sha256[:16]=$(sh_ $G/config.toml)  secrets/ entries: $(ls -A $G/secrets | tr '\n' ' ')"; }
cx() { (cd $S/fakehome && env -i HOME=$S/fakehome PATH=/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$G $B "$@"); }
ask() { cx exec --skip-git-repo-check "$@" "Reply with exactly the word pong and nothing else." 2>&1 | grep -E -i '^pong|error|ignored' | sort -u | sed 's/^/   | /'; }
echo "M0 home G created by PRE-PF-84 build 44b527f11f (onboarding stored the Z.AI key in the vault; pre build replied pong)"; snap
echo "M1 candidate, flag OFF (config untouched): exec"; ask; snap
echo "M2 candidate, flag ON via --enable: exec on default account"; ask --enable named_accounts; snap
echo "M3 candidate, flag ON: account list"; cx account list --enable named_accounts 2>&1 | sed 's/^/   | /'; snap
echo "M4 candidate, flag ON: explicit default selector"; ask --enable named_accounts -c provider_accounts.zai='"default"'; snap
echo "M5 first named write: add fake named account 'side'"; printf 'fake-side-%s' "$(openssl rand -hex 12)" | cx account add zai side --enable named_accounts 2>&1 | sed 's/^/   | /'; snap
echo "M6 default still resolves after the named write (flag on, then off)"; ask --enable named_accounts; ask
echo "M7 named side -> 401 (isolated from default)"; ask --enable named_accounts -c provider_accounts.zai='"side"'
echo "M8 PRE build still reads the home after the named write (downgrade safety)"; (cd $S/fakehome && env -i HOME=$S/fakehome PATH=/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$G $S/bin/corbanu-pre exec --skip-git-repo-check "Reply with exactly the word pong and nothing else." 2>&1) | grep -E -i '^pong|error' | sort -u | sed 's/^/   | /'
echo "M9 remove 'side'"; cx account remove zai side --enable named_accounts 2>&1 | sed 's/^/   | /'; cx account list --enable named_accounts 2>&1 | sed 's/^/   | /'; snap; ask --enable named_accounts
