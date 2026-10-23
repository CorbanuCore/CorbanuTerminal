#!/usr/bin/env bash
# Re-capture of the rs_suite cases whose first captures were empty (snap line-count bug). Same homes and ids.
set -u
source "$(dirname "$0")/drv.sh" fx
H=$S/homes; eval "$(cat $S/logs/rs-ids.txt)"; C_Z2=01a13456-fa03-70c0-9aa7-04f09aef40b2
vis() { screen $1 | sed "s#$S#\$S#g" > "$EV/captures/$2.txt"; echo "saved $2 (visible)"; }
nt r2 o2 resume $C_O2; sleep 12; snap r2 555-RS02-MSG03-missing-credentials 60; tx kill-server
nt r2p o2 resume; sleep 8; tx send-keys -t r2p Enter; sleep 10; snap r2p 555-RS04-d-o2-picker-enter 60; tx kill-server
nt r1p o1 resume; sleep 8; tx send-keys -t r1p Enter; sleep 12; vis r1p 555-RS04-b-o1-picker-enter; status r1p; snap r1p 555-RS04-b-o1-picker-enter-status 80; quit r1p; tx kill-server
zt r3 o3 resume $C_O3; sleep 12; snap r3 555-RS03-only-default 60; tx kill-server
nt e1 n1 resume; sleep 8; tx send-keys -t e1 Escape; sleep 3; snap e1 555-ESC01-b-after-esc 40; tx kill-server
nt e2 n1 resume; sleep 8; tx send-keys -t e2 C-c; sleep 3; snap e2 555-ESC02-after-ctrl-c 40; tx kill-server
zt g3 z2; sleep 10; snap g3 555-GH03-ghost-new-session 40; tx kill-server
zt g2 z2 resume $C_Z2; sleep 10; snap g2 555-GH02-ghost-and-work-missing 40; tx kill-server
zt g2b z2 resume $C_Z2 --account personal; sleep 12; status g2b; snap g2b 555-GH02b-control-explicit-account 80; quit g2b; tx kill-server
echo FIXUPS_DONE
