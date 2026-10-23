#!/usr/bin/env bash
# #555 resume cases RS-*, GH-*, MSG-*, ESC-*, PR-02 (homes from z_setup.sh). Captures: captures/555-*.txt
set -u
source "$(dirname "$0")/drv.sh" rs
killall_s() { for x in "$@"; do use $x; tx kill-server 2>/dev/null; done; }
H=$S/homes
vis() { screen $1 | sed "s#$S#\$S#g" > "$EV/captures/$2.txt"; echo "saved $2 (visible)"; }
sid() { cap $1 -80 | grep -E 'Session:' | tail -1 | awk '{print $3}'; }
# mkconv <session> <home> <starter: zt|nt> args... : one ping turn, prints the session id
mkconv() { local s=$1 h=$2 st=$3; shift 3; $st $s $h "$@" >&2; sleep 13; turn $s ping >&2; status $s >&2; local id=$(sid $s); snap $s 555-setup-conv-$h >&2; quit $s; tx kill-server; echo "$id"; }
exitcode() { cap $1 -20 | grep -oE 'EXITED-[0-9]+' | tail -1; }
ZAIW=(-c 'model_provider="zai"' -m glm-5.3-flash --account work)

# --- fresh homes o1 o2 o3 n1 (z1 keeps the FK evidence)
for h in o1 o2 o3 n1 z2; do [ -d $H/$h ] && rm -r $H/$h < /dev/null; done
zcfg $H/o1 openai; zenrol $H/o1 work; zcfg $H/o2 openai; zcfg $H/o3 openai; zcfg $H/n1 zai; zenrol $H/n1 work
for h in o1 o2 o3; do printf 'sk-canary-defaultkey-S6B' | run $CAND $H/$h login --with-api-key > /dev/null 2>&1; done
# --- conversations recorded on zai `work` in each home
C_O1=$(mkconv c1 o1 nt "${ZAIW[@]}"); echo "o1 conv: $C_O1"
# o2: enrol work, record, then remove work (H-OAI-bare)
zenrol $H/o2 work; C_O2=$(mkconv c2 o2 nt "${ZAIW[@]}"); run $CAND $H/o2 account remove zai work; echo "o2 conv: $C_O2"
# o3 (RS-03): openai home, zai default key only (ZAI_API_KEY in env), conversation recorded on work, then work removed
zenrol $H/o3 work; C_O3=$(mkconv c3 o3 nt "${ZAIW[@]}"); run $CAND $H/o3 account remove zai work; echo "o3 conv: $C_O3"
C_N1=$(mkconv c4 n1 nt --account work); echo "n1 conv: $C_N1"
echo "C_O1=$C_O1 C_O2=$C_O2 C_O3=$C_O3 C_N1=$C_N1" > $S/logs/rs-ids.txt

# RS-01 / RS-05 / REG-05: o1, credentials present
source "$(dirname "$0")/drv.sh" rs1
nt r1 o1 resume $C_O1; sleep 14; vis r1 555-RS01-a-after-launch; turn r1 pong; status r1; snap r1 555-RS01-b-resumed-status; quit r1
nt r5 o1 resume $C_O1; sleep 14; status r5; snap r5 555-RS05-resume-twice-status; quit r5
# RS-02 / MSG-03: o2, credentials missing
nt r2 o2 resume $C_O2; sleep 10; snap r2 555-RS02-MSG03-missing-credentials 60
# RS-03: o3, only zai default (real key in env)
zt r3 o3 resume $C_O3; sleep 12; snap r3 555-RS03-only-default 60
# RS-04: picker in o1 and o2
nt p1 o1 resume; sleep 8; vis p1 555-RS04-a-o1-picker
nt p2 o2 resume; sleep 8; vis p2 555-RS04-c-o2-picker

# ESC-01 / ESC-02: n1 (H-NOKEY)
source "$(dirname "$0")/drv.sh" esc
nt e1 n1 resume; sleep 8; vis e1 555-ESC01-a-picker; tx send-keys -t e1 Escape; sleep 3; snap e1 555-ESC01-b-after-esc 30
nt e2 n1 resume; sleep 8; tx send-keys -t e2 C-c; sleep 3; snap e2 555-ESC02-after-ctrl-c 30
# ESC-04: fresh home, no credentials, new session
[ -d $H/fresh ] && rm -r $H/fresh < /dev/null; mkdir -p $H/fresh; printf '[features]\nnamed_accounts = true\n[projects."%s/ws"]\ntrust_level = "trusted"\n' $S > $H/fresh/config.toml
nt e4 fresh; sleep 10; vis e4 555-ESC04-fresh-home-onboarding

# GH-01 / GH-03 / GH-02: z2 = copy of z1 with zai = "ghost"
source "$(dirname "$0")/drv.sh" gh
zcfg $H/z2 zai 'zai = "ghost"'; zenrol $H/z2 work personal; grep -A1 provider_accounts $H/z2/config.toml
C_Z2=$(mkconv c5 z2 zt --account work); echo "z2 conv: $C_Z2"
zt g1 z2 resume $C_Z2; sleep 14; turn g1 pong; status g1; snap g1 555-GH01-ghost-resume-on-work; quit g1
zt g3 z2; sleep 10; snap g3 555-GH03-ghost-new-session 40
run $CAND $H/z2 account remove zai work
zt g2 z2 resume $C_Z2; sleep 10; snap g2 555-GH02-ghost-and-work-missing 40

# PR-02: z1, resume with --account openai:x
source "$(dirname "$0")/drv.sh" pr2
zt q1 z1 resume $(cat $S/logs/fk5-conv.txt) --account openai:x; sleep 12; snap q1 555-PR02-a-launch 40
cap q1 -5 | grep -q EXITED || { turn q1 pong; status q1; snap q1 555-PR02-b-status; quit q1; }
killall_s r1 r5 r2 r3 p1 p2 e1 e2 e4 g1 g2 g3 q1
echo RS_SUITE_DONE
