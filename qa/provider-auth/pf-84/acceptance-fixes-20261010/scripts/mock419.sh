#!/bin/bash
S=<scratch>
python3 <scratch>/acc/qa/provider-auth/pf-84/independent-acceptance-20261010/scripts/mock.py 18485 $S/mock/requests.log $S/mock/canaries.json & MP=$!
sleep 1
for v in before after; do
  h=$S/hq-$v; rm -r $h 2>/dev/null </dev/null; mkdir -p $h
  sed -e "s#/Volumes/CorbanuDrive/Corbanu/tmp/pf84acc-scratch/mock/authcmd.sh#$S/mock/authcmd.sh#" -e 's/18484/18485/' <scratch>/acc/qa/provider-auth/pf-84/independent-acceptance-20261010/scripts/fixtures/hQ-mock-authcommand-provider.config.toml > $h/config.toml
  : > $S/mock/requests.log; : > $S/mock/authcmd.log
  (cd $S/work && env -i HOME=$S/fakehome PATH=/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$h /opt/homebrew/bin/gtimeout 90 $S/bin/corbanu-$v exec --skip-git-repo-check -c provider_accounts.cmdp='"ghost"' "x" < /dev/null > $S/mock/out-$v.txt 2>&1)
  echo "== $v: requests=$(wc -l < $S/mock/requests.log | tr -d ' ') ($(grep -c '"path": "/v1/models' $S/mock/requests.log) GET /models, $(grep -c '"bearer_id": "NONE"' $S/mock/requests.log) without Authorization) auth-command runs=$(wc -l < $S/mock/authcmd.log | tr -d ' ')"
  grep -E '^ERROR' $S/mock/out-$v.txt | sort -u | head -2 | cut -c1-300 | sed 's/^/   | /'
done
kill $MP
