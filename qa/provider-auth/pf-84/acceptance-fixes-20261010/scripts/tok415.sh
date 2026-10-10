#!/bin/bash
S=<scratch>
for v in before after; do
  for how in "--account fake --disable named_accounts" "--disable named_accounts (CORBANU_PROVIDER_ACCOUNT=fake)"; do
    args=${how% (*}; envacct=; [[ "$how" == *PROVIDER_ACCOUNT* ]] && envacct=CORBANU_PROVIDER_ACCOUNT=fake
    out=$(env -i HOME=$S/fakehome PATH=/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$S/hc $envacct $S/bin/corbanu-$v internal-claude-oauth-token $args 2>$S/tok.err); rc=$?
    sha=$( [ -n "$out" ] && printf '%s' "$out" | shasum -a 256 | cut -c1-12 || echo "<empty>")
    echo "== $v internal-claude-oauth-token $how: exit=$rc stdout-sha12=$sha"
    head -1 $S/tok.err | cut -c1-200 | sed 's/^/   | /'
  done
done
echo "   (sha12 of the fake account's token: $(printf 'sk-ant-''oat01-pf84fix-fake-not-a-token' | shasum -a 256 | cut -c1-12))"
