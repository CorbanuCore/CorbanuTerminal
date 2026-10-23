#!/usr/bin/env bash
# #555 fork cases FK-01..FK-09, ESC-03, PR-01 on home z1 (H-ZAI). Captures: captures/555-*.txt
set -u
source "$(dirname "$0")/drv.sh" fk
SRC=${SRC:-}
# FK-01 / FK-04 / FK-08 / FK-03 / ESC-03 / FK-09 in one session started on --account work
zt fk z1 --account work; sleep 12
turn fk ping; status fk
SRC=$(cap fk -60 | grep -E 'Session:' | tail -1 | awk '{print $3}'); echo "$SRC" > $S/logs/fk-src.txt
send_line fk "/fork"; wait_for fk 'Thread forked from' 20; turn fk forked; status fk; snap fk 555-FK01-fork-status
send_line fk "/side reply with the single word side"; wait_for fk '^• side$' 90; status fk
tx send-keys -t fk Escape; sleep 0.4; tx send-keys -t fk Escape; sleep 2; snap fk 555-FK04-side-status
tx send-keys -t fk Escape; sleep 1; tx send-keys -t fk 'C-/'; sleep 3; turn fk main; status fk; snap fk 555-FK04-main-after-side-status
send_line fk "/fork"; sleep 5; turn fk forktwo; status fk; snap fk 555-FK08-fork-of-fork-status
tx send-keys -t fk Escape; sleep 0.5; tx send-keys -t fk Escape; sleep 2; for i in 1 2 3; do tx send-keys -t fk Left; sleep 0.6; done
tx send-keys -t fk Enter; sleep 2; tx send-keys -t fk C-u; sleep 0.5; turn fk edited; status fk; snap fk 555-FK03-edit-prompt-status
send_line fk "/resume"; sleep 4; snap fk 555-ESC03-a-inapp-resume-picker 60; tx send-keys -t fk Escape; sleep 2; status fk; snap fk 555-ESC03-b-after-esc-status
send_line fk "Use your spawn agent tool to start exactly one sub-agent whose task is: reply with exactly the word child. Wait for it to finish, then reply with exactly the word done."
wait_for fk '^• done$' 150; send_line fk "/cost"; sleep 4; snap fk 555-FK09-spawn-and-cost
quit fk; tx kill-server
# FK-02 + PR-01 (no flag): resume the source without --account (z1 config says personal), then /fork
source "$(dirname "$0")/drv.sh" fk2
zt fk2 z1 -c 'model_reasoning_effort="high"' resume $SRC; sleep 14; status fk2
send_line fk2 "/fork"; wait_for fk2 'Thread forked from' 20; turn fk2 resfork; status fk2; snap fk2 555-FK02-PR01b-resume-then-fork-status
quit fk2; tx kill-server
# FK-05 / FK-06: corbanu fork <id> [--account ...]
for v in none personal zai:personal; do
  n=fk5$(echo $v | tr -d ':'); source "$(dirname "$0")/drv.sh" $n
  if [ $v = none ]; then zt $n z1 fork $SRC; else zt $n z1 fork $SRC --account $v; fi
  sleep 14; turn $n clifork; status $n; snap $n 555-FK0$([ $v = none ] && echo 5-cli-fork || echo 6-cli-fork-account-$(echo $v | tr ':' '-'))
  quit $n; tx kill-server
done
# PR-01 a/c/d
source "$(dirname "$0")/drv.sh" pr1
pr() { local n=$1 c=$2; shift 2; zt $n z1 "$@"; sleep 14; turn $n prec; status $n; snap $n $c; quit $n; }
pr pa 555-PR01a-resume-with-account-personal resume $SRC --account personal
pr pc 555-PR01c-new-session-noflag
cp $S/homes/z1/config.toml $S/logs/z1-config.bak; sed -i '' '/^\[provider_accounts\]/,/^zai = /d' $S/homes/z1/config.toml
pr pd 555-PR01d-new-session-without-provider-accounts
cp $S/logs/z1-config.bak $S/homes/z1/config.toml; tx kill-server
echo FK_SUITE_DONE
