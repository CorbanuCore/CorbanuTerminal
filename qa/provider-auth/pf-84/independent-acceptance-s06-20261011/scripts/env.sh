# Source me. Independent S06 acceptance environment (paths are this host's).
export S=${S:-/Volumes/CorbanuDrive/Corbanu/.codex-work/s06acc}
T=/Volumes/CorbanuDrive/Corbanu/.codex-work/targets
export CAND=$T/pf84-s06-indep-acc-20261011/debug/corbanu   # origin/main a22fdb903d, developer-accounting
export BASE=$T/pf84-s06-indep-base-20261011/debug/corbanu  # 3a0a38bed2 (pre-S06, first parent of #543), developer-accounting
export KH=/Users/Neo/.local/bin/corbanu.bak-20261010       # installed signed helper (real vault, read-only use)
export EV=${EV:-/Volumes/CorbanuDrive/Corbanu/worktrees/pf-84-s06-indep-accept-20261011/qa/provider-auth/pf-84/independent-acceptance-s06-20261011}
export SOCK=$S/tmux.sock
mkdir -p "$S/homes" "$S/fakehome" "$S/ws" "$S/logs" "$S/pathbin"

# Real key, consumed only by the command that follows (never printed).
#   key <label> cmd...   -> runs cmd with SECRET in its environment
key() { local label=$1; shift; SECRET="$(env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME "$KH" vault auth-helper "$label")" "$@"; }

# Run a binary in a disposable Corbanu home and a disposable HOME (FH), no keychain access.
# Extra variables (decoys, overrides) come from the XENV array.
#   run <bin> <corbanu-home> args...
run() {
  local bin=$1 home=$2; shift 2
  env -i HOME="${FH:-$S/fakehome}" USER="$USER" TERM=xterm-256color LANG=en_US.UTF-8 \
    PATH="$S/pathbin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin" \
    CORBANU_HOME="$home" CORBANU_TEST_NO_NATIVE_KEYRING=1 ${XENV[@]+"${XENV[@]}"} "$bin" "$@"
}
# Same, with $SECRET on stdin (account add / login --with-api-key). Never in argv.
run_secret_stdin() { printf '%s' "$SECRET" | run "$@"; }
# Print a command line, run it, print the exit code; output with fake-value redaction.
show() { printf '$ %s\n' "$*" | sed "s#$S#\$S#g"; "$@" 2>&1 | sed "s#$S#\$S#g"; printf '[exit %s]\n' "${PIPESTATUS[0]}"; }
mocklines() { wc -l < "$1" | tr -d ' '; }
