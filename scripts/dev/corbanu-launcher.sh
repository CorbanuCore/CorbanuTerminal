#!/bin/sh
# Corbanu dev launcher (installed as ~/.local/bin/corbanu on the Mac dev host).
# Mirrors /Applications/Corbanu Terminal Launcher.app: same stable build link
# and environment, so new builds are picked up automatically.
#
# PF-84-S01: a home the caller already chose (CORBANU_HOME, PFTERMINAL_HOME or
# CODEX_HOME) is kept. activate.sh exports the default home unconditionally, so
# without this a worker started as `CORBANU_HOME=<other> corbanu ...` silently
# ran on the default home and its account.
CORBANU_WORK_DIR="${CORBANU_LAUNCHER_WORK_DIR:-/Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal}"
CORBANU_WORKSPACE_DIR="${CORBANU_LAUNCHER_WORKSPACE_DIR:-/Volumes/CorbanuDrive/Corbanu}"
if ! cat "$CORBANU_WORK_DIR/activate.sh" >/dev/null 2>&1; then
  echo "corbanu: macOS is blocking access to CorbanuDrive." >&2
  if [ -n "$SSH_CONNECTION" ]; then
    echo "Over SSH: on this Mac, System Settings > General > Sharing > Remote Login (i)" >&2
    echo "> enable 'Allow full disk access for remote users', then reconnect." >&2
  else
    echo "Allow this terminal app in System Settings > Privacy & Security > Files and Folders" >&2
    echo "(Removable Volumes) or Full Disk Access, then restart the terminal app." >&2
  fi
  exit 1
fi
# Over SSH the login keychain (vault key, Claude login) is locked; unlock it
# via macOS's own password prompt so providers like claude-plan work.
if [ -n "$SSH_CONNECTION" ] && ! /usr/bin/security show-keychain-info login.keychain-db >/dev/null 2>&1; then
  echo "corbanu: unlocking your login keychain for this SSH session." >&2
  /usr/bin/security unlock-keychain login.keychain-db || exit 1
fi
caller_corbanu_home="${CORBANU_HOME:-}"
caller_pfterminal_home="${PFTERMINAL_HOME:-}"
caller_codex_home="${CODEX_HOME:-}"
. "$CORBANU_WORK_DIR/activate.sh"
if [ -n "$caller_corbanu_home$caller_pfterminal_home$caller_codex_home" ]; then
  # Restore exactly what the caller set; the binary warns if they disagree.
  restore_home() {
    if [ -n "$2" ]; then export "$1=$2"; else unset "$1"; fi
  }
  restore_home CORBANU_HOME "$caller_corbanu_home"
  restore_home PFTERMINAL_HOME "$caller_pfterminal_home"
  restore_home CODEX_HOME "$caller_codex_home"
fi
# Like the app launcher, start in the Corbanu workspace (where sessions live)
# when invoked from the home folder, e.g. right after an SSH login.
[ "$(pwd -P)" = "$(cd "$HOME" && pwd -P)" ] && cd "$CORBANU_WORKSPACE_DIR"
exec "$CORBANU_WORK_DIR/bin/corbanu" "$@"
