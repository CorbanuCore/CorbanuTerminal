#!/usr/bin/env bash
# Review round 1, finding 2: re-run the #555 charge checks in a home where only `work` holds the real Z.AI key.
# z4 = H-ZAI with distinct keys: work = REAL key, personal = FAKE key, default = FAKE key (ZAI_API_KEY). A request routed
# to the wrong account then fails with 401 at Z.AI instead of being counted by Corbanu only.
set -u
source "$(dirname "$0")/drv.sh" fk2s
H=$S/homes
sid() { cap $1 -80 | grep -E 'Session:' | tail -1 | awk '{print $3}'; }
lastlines() { cap $1 -40 | grep -E '^• |^■|^⚠|Model provider|Account usage|Forked from|Session:|EXITED' | cut -c1-200 | tail -${2:-8}; }
[ -d $H/z4 ] && rm -r $H/z4 < /dev/null; zcfg $H/z4 zai 'zai = "personal"'; zenrol $H/z4 work
SECRET=qa-fake-zai-personal.S6BFAKE run_secret_stdin $CAND $H/z4 account add zai personal
run $CAND $H/z4 account list
echo "### FK-01/FK-04/FK-08/FK-03/FK-09 on --account work (default and personal are fake)"
ft a z4 --account work; sleep 13; turn a ping; status a; SRC=$(sid a); echo "$SRC" > $S/logs/fk2-src.txt; echo "source: $SRC"
send_line a "/fork"; wait_for a 'Thread forked from' 20; turn a forked; status a; snap a 555r2-FK01-fork; lastlines a
send_line a "/side reply with the single word side"; wait_for a '^• side$' 90; status a; snap a 555r2-FK04-side; lastlines a 4
tx send-keys -t a Escape; sleep 1; tx send-keys -t a 'C-/'; sleep 3; turn a main; status a; snap a 555r2-FK04-main; lastlines a 4
send_line a "/fork"; sleep 5; turn a forktwo; status a; snap a 555r2-FK08-fork-of-fork; lastlines a 4
tx send-keys -t a Escape; sleep 0.5; tx send-keys -t a Escape; sleep 2; for i in 1 2 3; do tx send-keys -t a Left; sleep 0.6; done
tx send-keys -t a Enter; sleep 2; tx send-keys -t a C-u; sleep 0.5; turn a edited; status a; snap a 555r2-FK03-edit; lastlines a 4
send_line a "Use your spawn agent tool to start exactly one sub-agent whose task is: reply with exactly the word child. Wait for it to finish, then reply with exactly the word done."
wait_for a '^• done$' 150; send_line a "/cost"; sleep 4; snap a 555r2-FK09-spawn-cost; cap a -60 | grep -E 'Spawned|Completed|Z.AI · account|⚠' | cut -c1-200
quit a; tx kill-server
echo "### FK-02 / PR-01b: resume without a flag, then /fork"
ft b z4 resume $SRC; sleep 14; turn b resumed; status b; send_line b "/fork"; wait_for b 'Thread forked from' 20; turn b resfork; status b; snap b 555r2-FK02-PR01b; lastlines b 6; quit b; tx kill-server
echo "### FK-05 / FK-06"
ft c z4 fork $SRC; sleep 14; turn c clifork; status c; snap c 555r2-FK05; lastlines c 5; quit c; tx kill-server
ft d z4 fork $SRC --account zai:personal; sleep 14; turn d acctfork; status d; snap d 555r2-FK06-expect-401-on-personal; lastlines d 6; quit d; tx kill-server
echo "### PR-01 a/c/d"
ft e z4 resume $SRC --account personal; sleep 14; turn e prec; snap e 555r2-PR01a-expect-401-on-personal; lastlines e 5; quit e; tx kill-server
ft f z4; sleep 14; turn f prec; snap f 555r2-PR01c-expect-401-on-personal; lastlines f 5; quit f; tx kill-server
cp $H/z4/config.toml $S/logs/z4-config.bak; sed -i '' '/^\[provider_accounts\]/,/^zai = /d' $H/z4/config.toml
ft g z4; sleep 14; turn g prec; snap g 555r2-PR01d-expect-401-on-default; lastlines g 5; quit g; tx kill-server
cp $S/logs/z4-config.bak $H/z4/config.toml
echo "### GH-01: zai = ghost, resume the work conversation"
sed -i '' 's/^zai = "personal"/zai = "ghost"/' $H/z4/config.toml
ft h z4 resume $SRC; sleep 14; turn h pong; status h; snap h 555r2-GH01; lastlines h 5; quit h; tx kill-server
cp $S/logs/z4-config.bak $H/z4/config.toml
echo "### ESC-03 in one session: draft check, /resume picker, Esc, /status"
ft i z4 --account work; sleep 13; turn i ping; status i
send_line i "/resume"; sleep 4; screen i | sed "s#$S#\$S#g" > $EV/captures/555r2-ESC03-a-picker.txt; tx send-keys -t i Escape; sleep 2; status i; snap i 555r2-ESC03-b-after-esc; lastlines i 4; quit i; tx kill-server
echo FK2_DONE
