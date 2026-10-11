#!/bin/bash
S=<scratch>
for v in before after; do
  n=$(cd $S/work && env -i HOME=$S/fakehome PATH=/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$S/hw /opt/homebrew/bin/gtimeout 60 $S/bin/corbanu-$v exec --skip-git-repo-check -c provider_accounts.zai='"x"' "x" < /dev/null 2>&1 | grep -c 'is ignored because the')
  echo "== exec $v: $n '[provider_accounts] is ignored' warning line(s)"
done
