#!/usr/bin/env bash
# REG-06: #555 blocker subset on Linux (glitch), debug + developer-accounting build of 53c02c9301.
# Runs ON glitch under ~/corbanu-rtx/s06b-qa. The real Z.AI key arrives on stdin (account add only), never argv.
# usage (from the Mac): scp linux_glitch.sh glitch:corbanu-rtx/s06b-qa-run.sh; key provider/zai_api_key sh -c 'printf %s "$SECRET" | ssh glitch bash corbanu-rtx/s06b-qa-run.sh'
set -u
R=$HOME/corbanu-rtx/s06b-qa; BIN=$HOME/corbanu-rtx/s06b-target/debug/corbanu
KEY=$(cat /dev/stdin 2>/dev/null || true)
[ -d $R ] && rm -r $R < /dev/null; mkdir -p $R/fh $R/ws $R/out; SOCKD=$R/sock; mkdir -p $SOCKD
run() { local h=$1; shift; env -i HOME=$R/fh USER=$USER TERM=xterm-256color LANG=C.UTF-8 PATH=/usr/bin:/bin CORBANU_HOME=$h CORBANU_TEST_NO_NATIVE_KEYRING=1 $BIN "$@"; }
cfg() { mkdir -p $1; { echo 'model = "glm-5.3-flash"'; echo "model_provider = \"$2\""; echo 'approval_policy = "never"'; echo 'sandbox_mode = "danger-full-access"'
  echo 'suppress_unstable_features_warning = true'; echo '[features]'; echo 'named_accounts = true'; [ -n "${3:-}" ] && { echo '[provider_accounts]'; echo "$3"; }
  echo "[projects.\"$R/ws\"]"; echo 'trust_level = "trusted"'; } > $1/config.toml; }
T() { tmux -S $SOCKD/$1 "${@:2}"; }
tui() { local s=$1 h=$2; shift 2; local cmd; cmd=$(printf '%q ' env -i HOME=$R/fh USER=$USER TERM=xterm-256color LANG=C.UTF-8 PATH=/usr/bin:/bin CORBANU_HOME=$h CORBANU_TEST_NO_NATIVE_KEYRING=1 $BIN -C $R/ws "$@")
  T $s new-session -d -s $s -x 170 -y 50 "$cmd; echo EXITED-\$?; sleep 300"; }
scr() { T $1 capture-pane -p -t $1 -S -${2:-200} | sed "s#$R#\$R#g; s#$HOME#~#g"; }
say() { T $1 send-keys -t $1 -l -- "$2"; sleep 0.8; T $1 send-keys -t $1 Enter; }
waitre() { for i in $(seq 1 ${3:-90}); do scr $1 60 | grep -Eq -- "$2" && return 0; sleep 1; done; echo "TIMEOUT /$2/ in $1"; }
sid() { scr $1 80 | grep -E 'Session:' | tail -1 | awk '{print $3}'; }
quit() { T $1 send-keys -t $1 Escape; sleep 0.5; say $1 /exit; sleep 3; T $1 kill-server; }
Z=$R/z; cfg $Z zai 'zai = "personal"'
for a in work personal; do printf %s "$KEY" | run $Z account add zai $a; done
O=$R/o; cfg $O openai; N=$R/n; cfg $N zai
echo "## uname: $(uname -srm); binary: $($BIN --version 2>/dev/null | head -1)"
# conversation on work
tui c $Z --account work; sleep 12; say c "Reply with exactly the word ping"; waitre c '^• ping$'; say c /status; sleep 3; SRC=$(sid c); quit c; echo "source conversation: $SRC"
# FK-05: corbanu fork <id> -> work
tui f $Z fork $SRC; sleep 12; say f "Reply with exactly the word clifork"; waitre f '^• clifork$'; say f /status; sleep 3; echo "### FK-05 linux"; scr f 80 | grep -E 'forked from|^• |Model provider|Forked from|Account usage'; quit f
# FK-06: --account personal
tui f6 $Z fork $SRC --account personal; sleep 12; say f6 "Reply with exactly the word acctfork"; waitre f6 '^• acctfork$'; say f6 /status; sleep 3; echo "### FK-06 linux"; scr f6 80 | grep -E '^• |Model provider|Forked from|Account usage'; quit f6
# GH-01: ghost config, resume on work
sed -i 's/^zai = "personal"/zai = "ghost"/' $Z/config.toml
tui g1 $Z resume $SRC; sleep 12; say g1 "Reply with exactly the word pong"; waitre g1 '^• pong$'; say g1 /status; sleep 3; echo "### GH-01 linux"; scr g1 80 | grep -E '^• |Model provider|Session:|Account usage'; quit g1
# GH-03: new session with ghost
tui g3 $Z; sleep 10; echo "### GH-03 linux"; scr g3 20 | grep -v '^\s*$'; T g3 kill-server
# RS-02 / MSG-03 / GH-02: work removed
run $Z account remove zai work
tui g2 $Z resume $SRC; sleep 10; echo "### GH-02 / MSG-03 linux"; scr g2 20 | grep -v '^\s*$'; T g2 kill-server
# ESC-01: home without a default key, picker, Esc
printf %s "$KEY" | run $N account add zai work; tui nc $N --account work; sleep 12; say nc "Reply with exactly the word ping"; waitre nc '^• ping$'; quit nc
tui e1 $N resume; sleep 8; echo "### ESC-01 linux (picker)"; scr e1 20 | grep -v '^\s*$' | tail -6; T e1 send-keys -t e1 Escape; sleep 3; echo "### ESC-01 linux (after Esc)"; scr e1 20 | grep -v '^\s*$' | tail -3; T e1 kill-server
echo "### leak check: real key in any file under $R"; grep -rlF -f <(printf "%s\n" "$KEY") $R 2>/dev/null | sed "s#$R#\$R#g" | grep -v '^$' || echo "0 files"
unset KEY
echo LINUX_DONE
