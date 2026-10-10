#!/bin/sh
# PF-84-S01: the dev launcher keeps a caller-selected home and otherwise uses
# activate.sh's default. Runs against a stub work dir; no real binary or home.
set -eu
launcher="$(cd "$(dirname "$0")" && pwd)/corbanu-launcher.sh"
work="$(mktemp -d)"
trap 'rm -r "$work"' EXIT
mkdir -p "$work/bin" "$work/default-home"
cat >"$work/activate.sh" <<EOF
export CORBANU_HOME="$work/default-home"
export CODEX_HOME="\$CORBANU_HOME"
EOF
cat >"$work/bin/corbanu" <<'EOF'
#!/bin/sh
printf '%s|%s|%s\n' "${CORBANU_HOME:-}" "${PFTERMINAL_HOME:-}" "${CODEX_HOME:-}"
EOF
chmod +x "$work/bin/corbanu"

run() {
  env -i PATH=/usr/bin:/bin HOME="$work" CORBANU_WORK_DIR="$work" \
    CORBANU_WORKSPACE_DIR="$work" "$@" /bin/sh "$launcher"
}
check() {
  if [ "$1" != "$2" ]; then
    echo "FAIL: $3: got '$1', want '$2'" >&2
    exit 1
  fi
  echo "ok: $3"
}

check "$(run)" "$work/default-home||$work/default-home" "no caller home uses the default"
check "$(run CORBANU_HOME=/worker)" "/worker||" "caller CORBANU_HOME is kept"
check "$(run CODEX_HOME=/worker)" "||/worker" "caller CODEX_HOME is kept"
check "$(run CORBANU_HOME=/a CODEX_HOME=/b)" "/a||/b" "caller conflict is passed through for the binary to report"
