#!/usr/bin/env bash
# REG-01 / C1 in the TUI: named_accounts OFF (--disable named_accounts), candidate vs BASE (82b1b2cf32, main before
# #561/#566/#604/#610). FK-01, RS-01, ESC-01, ESC-03 and the /providers list.
set -u
source "$(dirname "$0")/drv.sh" off
H=$S/homes; eval "$(cat $S/logs/rs-ids.txt)"
vis() { screen $1 | sed "s#$S#\$S#g" > "$EV/captures/$2.txt"; echo "saved $2 (visible)"; }
for which in CAND BASE; do
  if [ $which = CAND ]; then export BIN=$CAND; else export BIN=$BASE; fi; t=$(echo $which | tr A-Z a-z)
  # FK-01 + ESC-03 + /providers (z1, default Z.AI key in env)
  zt f$t z1 --disable named_accounts; sleep 13; turn f$t ping; send_line f$t "/fork"; sleep 5; turn f$t forked; status f$t
  send_line f$t "/resume"; sleep 4; vis f$t 555-REG01-$t-ESC03-picker; tx send-keys -t f$t Escape; sleep 2; status f$t
  snap f$t 555-REG01-$t-FK01-ESC03-flag-off 200
  send_line f$t "/providers"; sleep 4; vis f$t tB-12-flag-off-providers-$t; tx send-keys -t f$t Escape; quit f$t; tx kill-server
  # RS-01 (o1: openai home, conversation recorded on zai work)
  nt r$t o1 --disable named_accounts resume $C_O1; sleep 14; snap r$t 555-REG01-$t-RS01-a-launch 80
  cap r$t -5 | grep -q EXITED || { turn r$t pong; status r$t; snap r$t 555-REG01-$t-RS01-b-status 80; quit r$t; }; tx kill-server
  # ESC-01 (n1: zai home, no default key)
  nt e$t n1 --disable named_accounts resume; sleep 8; vis e$t 555-REG01-$t-ESC01-a-picker; tx send-keys -t e$t Escape; sleep 4; snap e$t 555-REG01-$t-ESC01-b-after-esc 60; tx kill-server
done
echo REGOFF_DONE
