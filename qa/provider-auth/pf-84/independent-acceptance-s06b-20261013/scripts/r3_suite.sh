#!/usr/bin/env bash
# Review round 2: (1) GH-01 scripted in z4 with the config recorded in the capture; (2) Developer accounting under a
# clean Aggressive state (saved through /security, then restart), CAND and BASE, default and named Z.AI accounts, with
# the codex_core::accounting log lines.
set -u
source "$(dirname "$0")/drv.sh" r3
H=$S/homes
lastlines() { cap $1 -40 | grep -E '^• |^■|^⚠|Model provider|Account usage|Security:|Session:|EXITED' | cut -c1-200 | tail -${2:-8}; }
if [ "${PART:-all}" != 2 ]; then
# (1) GH-01
C=01a1347e-b31e-7f10-9248-4b064f96d4c4
sed -i '' 's/^zai = "personal"/zai = "ghost"/' $H/z4/config.toml
{ echo "## z4 config.toml for this run (work = REAL key; personal and default = FAKE keys)"; sed "s#$S#\$S#g" $H/z4/config.toml; echo "## account list"; run $CAND $H/z4 account list; echo "## conversation $C: its last turn ran on work (FK-05 fork, answered clifork, then pong)"; } > $EV/captures/555r3-GH01-config.txt
ft h z4 resume $C; sleep 14; turn h ghostok; status h; snap h 555r3-GH01-ghost-resume; lastlines h 5; quit h; tx kill-server
cp $S/logs/z4-config.bak $H/z4/config.toml
fi
# (2) accounting under Aggressive
aggr() { # aggr <bin-label> <bin> <home>
  local t=$1 bin=$2 h=$3; [ -d $H/$h ] && rm -r $H/$h < /dev/null; zcfg $H/$h zai; zenrol $H/$h work
  export BIN=$bin
  zt a1$t $h --enable security_levels; sleep 14; send_line a1$t "/security"; sleep 4; tx send-keys -t a1$t Down; sleep 0.4; tx send-keys -t a1$t Down; sleep 0.4
  tx send-keys -t a1$t Enter; sleep 3; tx send-keys -t a1$t Enter; sleep 3; screen a1$t | grep -E 'Saved|Restart' | head -2 | sed "s#^#   #"; tx send-keys -t a1$t Escape; sleep 1; quit a1$t; tx kill-server
  ls $H/$h | grep security_level
  local M=$(date -u +%s)
  zt a2$t $h --enable security_levels; sleep 14; turn a2$t pingdefault; status a2$t; snap a2$t tB-20-$t-aggressive-default; lastlines a2$t 6; quit a2$t; tx kill-server
  zt a3$t $h --enable security_levels --account work; sleep 14; turn a3$t pingwork; status a3$t; snap a3$t tB-20-$t-aggressive-named-work; lastlines a3$t 6; quit a3$t; tx kill-server
  { echo "## $t: codex_core::accounting WARN/ERROR lines since the two Aggressive sessions started (home $h)"; sqlite3 $H/$h/pfterminal_logs_2.sqlite "select datetime(ts,'unixepoch'), level, substr(feedback_log_body, instr(feedback_log_body,'}: ')+3, 200) from logs where target like '%accounting%' and level in ('WARN','ERROR') and ts >= $M order by ts;" | sed 's/.*stream_chat_completions_api{[^}]*}: //'; } > $EV/captures/tB-20-$t-accounting-log.txt
  cat $EV/captures/tB-20-$t-accounting-log.txt
  # back to Permissive through /security
  zt a4$t $h --enable security_levels; sleep 14; send_line a4$t "/security"; sleep 4; tx send-keys -t a4$t Enter; sleep 3; tx send-keys -t a4$t Enter; sleep 2; tx send-keys -t a4$t Escape; quit a4$t; tx kill-server
}
aggr cand $CAND z5
aggr base $BASE z6
unset BIN
echo R3_DONE
