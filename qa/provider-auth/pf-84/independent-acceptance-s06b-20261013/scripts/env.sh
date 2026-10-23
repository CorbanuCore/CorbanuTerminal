# Source me. Independent S06 slice B re-acceptance environment (paths are this host's).
export S=${S:-/Volumes/CorbanuDrive/Corbanu/.codex-work/s06bacc}
T=/Volumes/CorbanuDrive/Corbanu/.codex-work/targets
export CAND=$S/bin/corbanu-cand   # origin/main 53c02c9301, debug + developer-accounting
export BASE=$S/bin/corbanu-base   # 82b1b2cf32 (main before #561/#566/#604/#610), debug + developer-accounting
export KH=/Users/Neo/.local/bin/corbanu.bak-20261010       # installed signed helper (real vault, read-only use)
export EV=${EV:-/Volumes/CorbanuDrive/Corbanu/worktrees/s06b-acceptance-20261013/qa/provider-auth/pf-84/independent-acceptance-s06b-20261013}
export SOCK=${SOCK:-$S/tmux.sock}
mkdir -p "$S/homes" "$S/fakehome" "$S/ws" "$S/logs" "$S/pathbin"

# Real key, consumed only by the command that follows (never printed).
#   key <label> cmd...   -> runs cmd with SECRET in its environment
key() { local label=$1; shift; SECRET="$(env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME "$KH" vault auth-helper "$label")" "$@"; }

# Run a binary in a disposable Corbanu home and a disposable HOME (FH), no keychain access.
#   run <bin> <corbanu-home> args...   (extra variables from the XENV array)
run() {
  local bin=$1 home=$2; shift 2
  env -i HOME="${FH:-$S/fakehome}" USER="$USER" TERM=xterm-256color LANG=en_US.UTF-8 \
    PATH="$S/pathbin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin" \
    CORBANU_HOME="$home" CORBANU_TEST_NO_NATIVE_KEYRING=1 ${XENV[@]+"${XENV[@]}"} "$bin" "$@"
}
run_secret_stdin() { printf '%s' "$SECRET" | run "$@"; }
show() { printf '$ %s\n' "$*" | sed "s#$S#\$S#g; s#$CAND#\$CAND#g; s#$BASE#\$BASE#g"; "$@" 2>&1 | sed "s#$S#\$S#g"; printf '[exit %s]\n' "${PIPESTATUS[0]}"; }
mocklines() { wc -l < "$1" | tr -d ' '; }
