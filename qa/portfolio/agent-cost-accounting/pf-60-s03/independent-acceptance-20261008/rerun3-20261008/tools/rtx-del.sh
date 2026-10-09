#!/bin/bash
# remote side. usage: rtx-del.sh <home> <faketime|-> <tag> <session-uuid>: `corbanu delete --force` with a PLACEHOLDER key; optional faked clock
R=<rtx-dir>
h=$1; ft=$2; tag=$3; id=$4
pre=(); [ "$ft" != "-" ] && pre=(env TZ=UTC FAKETIME_DONT_FAKE_MONOTONIC=1 FAKETIME="@$ft" LD_PRELOAD=$R/libfaketime/src/libfaketime-time64.so.1)
echo "--- rollouts for $id before:"; find $h/sessions -name "*$id*" 2>/dev/null | sed "s#$R#<rtx-dir>#"
echo "\$ ${ft:+FAKETIME=@$ft }corbanu delete --force $id"
"${pre[@]}" env RUST_LOG=warn ZAI_API_KEY=placeholder-not-a-key CORBANU_TEST_NO_NATIVE_KEYRING=1 CODEX_HOME=$h CORBANU_HOME=$h PFTERMINAL_HOME=$h \
  ${BIN:-$R/corbanu-acct} delete --force "$id" < /dev/null > $R/logs/$tag.out 2> $R/logs/$tag.err
rc=$?
sed "s#$R#<rtx-dir>#g" $R/logs/$tag.out; sed "s#$R#<rtx-dir>#g" $R/logs/$tag.err | tail -20
echo "exit=$rc real_end=$(date -u +%FT%TZ)"
echo "--- rollouts for $id after:"; find $h/sessions -name "*$id*" 2>/dev/null | sed "s#$R#<rtx-dir>#"
