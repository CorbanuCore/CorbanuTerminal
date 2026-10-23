#!/usr/bin/env bash
# #555 MSG-01/02/04 and ESC-03 picker capture, home z3 (H-ZAI-like: [provider_accounts] zai = "personal").
set -u
source "$(dirname "$0")/drv.sh" msg
H=$S/homes
vis() { screen $1 | sed "s#$S#\$S#g" > "$EV/captures/$2.txt"; echo "saved $2 (visible)"; }
sid() { cap $1 -80 | grep -E 'Session:' | tail -1 | awk '{print $3}'; }
[ -d $H/z3 ] && rm -r $H/z3 < /dev/null; zcfg $H/z3 zai 'zai = "personal"'; zenrol $H/z3 work personal
zt c6 z3 --account work; sleep 13; turn c6 ping; status c6; CW=$(sid c6); quit c6; tx kill-server; echo "z3 work conv: $CW"
run $CAND $H/z3 account remove zai work
zt m z3; sleep 13; turn m first; status m; ORIG=$(sid m); echo "session before: $ORIG"
send_line m "/resume"; sleep 4; vis m 555-ESC03-a-inapp-resume-picker
# select the conversation recorded on work (the older one: second row)
tx send-keys -t m Down; sleep 0.5; vis m 555-MSG01-a-picker-selection; tx send-keys -t m Enter; sleep 6; snap m 555-MSG01-b-missing-account-message 80
# MSG-04: the original session still works
turn m after; status m; snap m 555-MSG04-session-after-failed-resume 80
# MSG-02: add `work` back through /providers (real key pasted from a tmux buffer, never argv), then /resume it again
send_line m "/providers"; sleep 4
for i in $(seq 1 30); do screen m | grep -E '^› +[0-9]+\. ' | tail -1 | grep -q 'Z.AI' && break; tx send-keys -t m Down; sleep 0.25; done
tx send-keys -t m Enter; sleep 2; vis m 555-MSG02-a-zai-actions
for i in $(seq 1 10); do screen m | grep -E '^› +[0-9]+\. ' | tail -1 | grep -q 'Add another account' && break; tx send-keys -t m Down; sleep 0.25; done
tx send-keys -t m Enter; sleep 2; tx send-keys -t m -l work; sleep 0.5; tx send-keys -t m Enter; sleep 2; vis m 555-MSG02-b-key-entry
key provider/zai_api_key sh -c 'printf %s "$SECRET" | /opt/homebrew/bin/tmux -S "$0" load-buffer -b k -' "$SOCK"; tx paste-buffer -b k -d -t m; sleep 1; vis m 555-MSG02-c-key-masked
tx send-keys -t m Enter; sleep 4; vis m 555-MSG02-d-saved; tx send-keys -t m Escape; sleep 1
send_line m "/resume"; sleep 4; tx send-keys -t m Down; sleep 0.5; tx send-keys -t m Enter; sleep 8
turn m pong; status m; snap m 555-MSG02-e-resumed-on-work-status 120
quit m; tx kill-server
echo MSG_DONE
