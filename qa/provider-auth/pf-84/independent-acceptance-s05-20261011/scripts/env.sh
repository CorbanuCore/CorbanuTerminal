# Source me. Independent S05 acceptance environment (paths are this host's).
export S=${S:-/Volumes/CorbanuDrive/Corbanu/.codex-work/s05acc}
T=/Volumes/CorbanuDrive/Corbanu/.codex-work/targets
export CAND=$T/pf84-s05-indep-acc-20261011/debug/corbanu        # origin/main d93f5e639d, developer-accounting
export BASE=$T/pf84-s05-indep-base-20261011/debug/corbanu       # bddf6292c4 (pre-S05), developer-accounting
export REL=$T/pf84-s05-indep-rel-20261011/release/corbanu       # origin/main d93f5e639d, --release, default features
export KH=/Users/Neo/.local/bin/corbanu.bak-20261010            # installed signed helper (real vault, read-only use)
export EV=${EV:-/Volumes/CorbanuDrive/Corbanu/worktrees/pf84-s05-indep-acc-20261011/qa/provider-auth/pf-84/independent-acceptance-s05-20261011}
export SOCK=$S/tmux.sock
mkdir -p "$S/homes" "$S/fakehome" "$S/ws" "$S/logs" "$S/pathbin"

# Real key, consumed only by the command that follows (never printed).
#   key provider/zai_api_key cmd...   -> runs cmd with SECRET in its environment
key() { local label=$1; shift; SECRET="$(env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME "$KH" vault auth-helper "$label")" "$@"; }

# Run a binary in a disposable home with no keychain access.
#   run <bin> <home> args...
run() {
  local bin=$1 home=$2; shift 2
  env -i HOME="$S/fakehome" USER="$USER" TERM=xterm-256color LANG=en_US.UTF-8 \
    PATH="$S/pathbin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin" \
    CORBANU_HOME="$home" CORBANU_TEST_NO_NATIVE_KEYRING=1 ${EXTRA_ENV:-} "$bin" "$@"
}
# Same, reading SECRET from the environment into stdin (for `account add`).
run_secret_stdin() { printf '%s' "$SECRET" | run "$@"; }
