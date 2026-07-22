#!/usr/bin/env bash
set -euo pipefail

: "${HOME:?HOME is required}"

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
CODEX_RS_DIR="$(cd -- "$SCRIPT_DIR/.." && pwd -P)"
TOKEN_ENV_VAR="PFTERMINAL_TELEGRAM_TOKEN"
DEFAULT_ENV_FILE="$HOME/.config/pfterminal/telegram.env"
DEFAULT_WORKSPACE="$HOME/pfterminal-telegram"

usage() {
    cat <<'USAGE'
Usage: setup-telegram.sh [OPTIONS]

Options:
  --chat-id ID          Allowed chat ID. Repeat or pass comma-separated IDs.
  --user-id ID          User allowed to act in group chats. Repeat or comma-separate.
  --workspace DIR       Telegram default_cwd. Defaults to ~/pfterminal-telegram.
  --env-file PATH       EnvironmentFile to write. Defaults to ~/.config/pfterminal/telegram.env.
  --approval-policy VAL Approval policy. Defaults to on-request.
  --allow-danger-full-access
                        Allow writing top-level sandbox_mode="danger-full-access"
                        after sandbox preflight failure. This disables the sandbox globally.
  --install-systemd     Install the systemd --user service template.
  --install-launchd     Install the macOS LaunchAgent template.
  --no-token-required   Skip token prompt/write for dry runs.
  -h, --help            Show this help.
USAGE
}

die() {
    printf 'error: %s\n' "$*" >&2
    exit 1
}

need_python() {
    command -v python3 >/dev/null 2>&1 || die "python3 is required"
}

abs_dir() {
    mkdir -p -- "$1"
    (cd -- "$1" && pwd -P)
}

abs_file() {
    local dir
    local base
    dir="$(dirname -- "$1")"
    base="$(basename -- "$1")"
    dir="$(abs_dir "$dir")"
    printf '%s/%s\n' "$dir" "$base"
}

default_codex_home_dir_name() {
    local source_file="$CODEX_RS_DIR/utils/home-dir/src/lib.rs"
    local extracted=""
    if [[ -r "$source_file" ]]; then
        extracted="$(sed -n 's/^const DEFAULT_PFTERMINAL_HOME_DIR: &str = "\(.*\)";/\1/p' "$source_file" | head -n 1)"
    fi
    [[ -n "$extracted" ]] && printf '%s\n' "$extracted" || printf '.pfterminal\n'
}

resolve_codex_home() {
    if [[ -n "${CODEX_HOME:-}" ]]; then
        abs_dir "$CODEX_HOME"
    else
        abs_dir "$HOME/$(default_codex_home_dir_name)"
    fi
}

CHAT_IDS=()
USER_IDS=()

add_chat_ids() {
    local value=$1
    local old_ifs=$IFS
    local raw
    local id
    local -a parts
    IFS=',' read -r -a parts <<< "$value"
    IFS=$old_ifs
    for raw in "${parts[@]}"; do
        id="${raw//[[:space:]]/}"
        [[ -z "$id" ]] && continue
        [[ "$id" =~ ^-?[0-9]+$ ]] || die "invalid Telegram chat ID: $raw"
        CHAT_IDS+=("$id")
    done
}

add_user_ids() {
    local value=$1 raw id
    local old_ifs=$IFS
    local -a parts
    IFS=',' read -r -a parts <<< "$value"
    IFS=$old_ifs
    for raw in "${parts[@]}"; do
        id="${raw//[[:space:]]/}"
        [[ -z "$id" ]] && continue
        [[ "$id" =~ ^[0-9]+$ ]] || die "invalid Telegram user ID: $raw"
        USER_IDS+=("$id")
    done
}

read_existing_ids() {
    [[ -f "$1" ]] || return 0
    python3 - "$1" "$2" "$3" <<'PY'
import sys
from pathlib import Path

try:
    import tomllib
except ModuleNotFoundError:
    try:
        import tomli as tomllib
    except ModuleNotFoundError:
        tomllib = None

contents = Path(sys.argv[1]).read_text()
key = sys.argv[2]
number_pattern = sys.argv[3]
if tomllib is not None:
    try:
        ids = tomllib.loads(contents).get("telegram", {}).get(key)
    except Exception as exc:
        print(f"failed to parse existing {key}: {exc}", file=sys.stderr)
        sys.exit(1)
else:
    import re

    ids = []
    in_telegram = False
    capturing = False
    buffer = ""
    for raw in contents.splitlines():
        line = raw.split("#", 1)[0].strip()
        if not line:
            continue
        if line.startswith("[") and line.endswith("]"):
            in_telegram = line == "[telegram]"
            capturing = False
            buffer = ""
            continue
        if not in_telegram:
            continue
        if capturing:
            buffer += line
        elif line.startswith(key) and "=" in line:
            buffer = line.split("=", 1)[1]
            capturing = True
        if capturing and "]" in buffer:
            ids = [int(value) for value in re.findall(number_pattern, buffer)]
            break

if ids:
    print(",".join(str(int(item)) for item in ids))
PY
}

read_existing_sandbox_mode() {
    [[ -f "$1" ]] || return 0
    python3 - "$1" <<'PY'
import sys
from pathlib import Path

try:
    import tomllib
except ModuleNotFoundError:
    try:
        import tomli as tomllib
    except ModuleNotFoundError:
        tomllib = None

try:
    contents = Path(sys.argv[1]).read_text()
    if tomllib is not None:
        sandbox_mode = tomllib.loads(contents).get("sandbox_mode")
    else:
        import re

        match = re.search(r'(?m)^\s*sandbox_mode\s*=\s*"([^"]+)"', contents)
        sandbox_mode = match.group(1) if match else None
except Exception:
    sys.exit(0)

if sandbox_mode:
    print(sandbox_mode)
PY
}

sandbox_issue() {
    [[ "$(uname -s)" == "Linux" ]] || return 0
    if ! command -v bwrap >/dev/null 2>&1; then
        printf 'bwrap was not found on PATH'
        return 0
    fi
    local value
    if [[ -r /proc/sys/user/max_user_namespaces ]]; then
        value="$(tr -d '[:space:]' < /proc/sys/user/max_user_namespaces)"
        if [[ "$value" == "0" ]]; then
            printf '/proc/sys/user/max_user_namespaces reads as 0'
            return 0
        fi
    fi
    if [[ -r /proc/sys/kernel/apparmor_restrict_unprivileged_userns ]]; then
        value="$(tr -d '[:space:]' < /proc/sys/kernel/apparmor_restrict_unprivileged_userns)"
        if [[ "$value" == "1" ]]; then
            printf '/proc/sys/kernel/apparmor_restrict_unprivileged_userns reads as 1'
            return 0
        fi
    fi
    if [[ -r /proc/sys/kernel/unprivileged_userns_clone ]]; then
        value="$(tr -d '[:space:]' < /proc/sys/kernel/unprivileged_userns_clone)"
        if [[ "$value" == "0" ]]; then
            printf '/proc/sys/kernel/unprivileged_userns_clone reads as 0'
            return 0
        fi
    fi
    if ! bwrap --ro-bind / / true >/dev/null 2>&1; then
        printf 'bwrap live probe failed: bwrap --ro-bind / / true'
        return 0
    fi
}

env_file_has_token() {
    [[ -f "$1" ]] || return 1
    grep -Eq "^${TOKEN_ENV_VAR}=.+" "$1"
}

write_env_file() {
    local token_action=preserve
    [[ -n "$3" ]] && token_action=set
    mkdir -p -- "$(dirname -- "$1")"
    if [[ -e "$1" ]]; then
        chmod 600 -- "$1"
    else
        install -m 600 /dev/null "$1"
    fi
    python3 - "$1" "$2" "$token_action" "$3" "$TOKEN_ENV_VAR" <<'PY'
import re
import sys
from pathlib import Path

path = Path(sys.argv[1])
updates = {"CODEX_HOME": sys.argv[2]}
if sys.argv[3] == "set":
    updates[sys.argv[5]] = sys.argv[4]

def quote(value):
    if "\n" in value:
        raise SystemExit("environment values must not contain newlines")
    return '"' + value.replace("\\", "\\\\").replace('"', '\\"') + '"'

key_re = re.compile(r"^([A-Za-z_][A-Za-z0-9_]*)=")
lines = path.read_text().splitlines() if path.exists() else []
out = []
seen = set()
for line in lines:
    match = key_re.match(line)
    if match and match.group(1) in updates:
        key = match.group(1)
        out.append(f"{key}={quote(updates[key])}")
        seen.add(key)
    else:
        out.append(line)
for key, value in updates.items():
    if key not in seen:
        out.append(f"{key}={quote(value)}")
path.write_text("\n".join(out) + "\n")
PY
}

backup_config() {
    [[ -f "$1" ]] || return 0
    local backup_path="$1.bak.$(date +%Y%m%d%H%M%S)"
    cp -p -- "$1" "$backup_path"
    printf 'Backed up existing config to %s\n' "$backup_path"
}

merge_config() {
    local config_path=$1
    local workspace=$2
    local sandbox_mode=$3
    local approval_policy=$4
    local explicit_keys=$5
    local chat_ids_csv=$6
    local user_ids_csv=$7
    python3 - "$config_path" "$workspace" "$sandbox_mode" "$approval_policy" "$explicit_keys" "$chat_ids_csv" "$user_ids_csv" <<'PY'
import json
import re
import sys
from pathlib import Path

path = Path(sys.argv[1])
explicit_keys = {key for key in sys.argv[5].split(",") if key}
candidates = {
    "enabled": True,
    "bot_token_env": "PFTERMINAL_TELEGRAM_TOKEN",
    "allowed_chat_ids": [int(value) for value in sys.argv[6].split(",") if value],
    "allowed_user_ids": [int(value) for value in sys.argv[7].split(",") if value],
    "mode": "polling",
    "approval_policy": sys.argv[4],
    "default_cwd": sys.argv[2],
}
order = list(candidates)
top = {"sandbox_mode": sys.argv[3]} if sys.argv[3] else {}
header_re = re.compile(r"^\s*\[\[?([^\]]+)\]\]?\s*(?:#.*)?$")
key_re = re.compile(r"^(\s*)([A-Za-z0-9_-]+)(\s*=).*$")

def fmt(value):
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, str):
        return json.dumps(value)
    return "[" + ", ".join(str(item) for item in value) + "]"

def section(line):
    match = header_re.match(line)
    return match.group(1) if match else None

def skip(lines, index):
    rest = lines[index].split("=", 1)[1] if "=" in lines[index] else ""
    depth = rest.count("[") - rest.count("]")
    index += 1
    while depth > 0 and index < len(lines):
        depth += lines[index].count("[") - lines[index].count("]")
        index += 1
    return index

def apply_range(lines, start, end, values, key_order):
    out = []
    seen = set()
    index = start
    while index < end:
        match = key_re.match(lines[index])
        if match and match.group(2) in values:
            key = match.group(2)
            out.append(f"{match.group(1)}{key} = {fmt(values[key])}")
            seen.add(key)
            index = skip(lines, index)
        else:
            out.append(lines[index])
            index += 1
    missing = [key for key in key_order if key in values and key not in seen]
    out.extend(f"{key} = {fmt(values[key])}" for key in missing)
    return out, missing

def table_range(lines, name):
    for start, line in enumerate(lines):
        if section(line) == name:
            end = next((i for i in range(start + 1, len(lines)) if section(lines[i]) is not None), len(lines))
            return start, end
    return None

def keys_in_range(lines, start, end):
    keys = set()
    index = start
    while index < end:
        match = key_re.match(lines[index])
        if match:
            keys.add(match.group(2))
            index = skip(lines, index)
        else:
            index += 1
    return keys

lines = (path.read_text() if path.exists() else "").splitlines()
if top:
    first_table = next((i for i, line in enumerate(lines) if section(line) is not None), len(lines))
    head, missing = apply_range(lines, 0, first_table, top, list(top))
    if missing and first_table < len(lines):
        head.append("")
    lines = head + lines[first_table:]

table = table_range(lines, "telegram")
if table is None:
    updates = dict(candidates)
    if lines and lines[-1].strip():
        lines.append("")
    lines.append("[telegram]")
    lines.extend(f"{key} = {fmt(updates[key])}" for key in order)
else:
    start, end = table
    existing_keys = keys_in_range(lines, start + 1, end)
    updates = {
        key: value
        for key, value in candidates.items()
        if key in explicit_keys or key not in existing_keys
    }
    body, _missing = apply_range(lines, start + 1, end, updates, order)
    lines = lines[: start + 1] + body + lines[end:]

output = "\n".join(lines) + "\n"
try:
    import tomllib
except ModuleNotFoundError:
    try:
        import tomli as tomllib
    except ModuleNotFoundError:
        print("validation skipped: Python 3.11+ tomllib or tomli is required", file=sys.stderr)
        tomllib = None
except Exception as exc:
    print(f"refusing to write invalid TOML: {exc}", file=sys.stderr)
    sys.exit(1)
if tomllib is not None:
    try:
        tomllib.loads(output)
    except Exception as exc:
        print(f"refusing to write invalid TOML: {exc}", file=sys.stderr)
        sys.exit(1)
path.parent.mkdir(parents=True, exist_ok=True)
path.write_text(output)
PY
}

seed_agents_md() {
    local target="$1/AGENTS.md"
    local template="$CODEX_RS_DIR/telegram/dist/AGENTS.md.template"
    if [[ -e "$target" ]]; then
        printf 'AGENTS.md already exists at %s; leaving it unchanged.\n' "$target"
        return
    fi
    [[ -r "$template" ]] || die "missing AGENTS.md template: $template"
    local content
    content="$(< "$template")"
    printf '%s\n' "${content//<cwd>/$1}" > "$target"
    printf 'Seeded Telegram identity instructions at %s\n' "$target"
}

install_systemd_unit() {
    local source="$CODEX_RS_DIR/telegram/dist/pfterminal-telegram.service"
    local target_dir="$HOME/.config/systemd/user"
    local target="$target_dir/pfterminal-telegram.service"
    local pfterminal_bin
    [[ -r "$source" ]] || die "missing systemd service template: $source"
    pfterminal_bin="$(command -v pfterminal || true)"
    [[ -n "$pfterminal_bin" ]] || die "pfterminal was not found on PATH; install it before --install-systemd"
    [[ "$pfterminal_bin" == /* ]] || die "command -v pfterminal did not return an absolute path: $pfterminal_bin"
    mkdir -p -- "$target_dir"
    python3 - "$source" "$target" "$pfterminal_bin" "$ENV_FILE" <<'PY'
import sys
from pathlib import Path

source = Path(sys.argv[1])
target = Path(sys.argv[2])
pfterminal_bin = sys.argv[3]
env_file = sys.argv[4]

lines = source.read_text().splitlines()
out = []
for line in lines:
    if line.startswith("ExecStart="):
        out.append(f"ExecStart={pfterminal_bin} telegram")
    elif line.startswith("EnvironmentFile="):
        out.append(f"EnvironmentFile={env_file}")
    else:
        out.append(line)
target.write_text("\n".join(out) + "\n")
PY
    printf 'Installed systemd user unit at %s\n' "$target"
    printf 'Enable it with:\n  systemctl --user daemon-reload\n  systemctl --user enable --now pfterminal-telegram.service\n'
}

install_launchd_unit() {
    [[ "$(uname -s)" == "Darwin" ]] || die "--install-launchd requires macOS"
    local source="$CODEX_RS_DIR/telegram/dist/net.postfiat.pfterminal.telegram.plist"
    local target_dir="$HOME/Library/LaunchAgents"
    local log_dir="$HOME/Library/Logs/PFTerminal"
    local target="$target_dir/net.postfiat.pfterminal.telegram.plist"
    local pfterminal_bin
    [[ -r "$source" ]] || die "missing launchd template: $source"
    pfterminal_bin="$(command -v pfterminal || true)"
    [[ "$pfterminal_bin" == /* ]] || die "pfterminal was not found at an absolute PATH entry"
    mkdir -p -- "$target_dir" "$log_dir"
    python3 - "$source" "$target" "$pfterminal_bin" "$CODEX_HOME_RESOLVED" "$log_dir" "$ENV_FILE" <<'PY'
import sys
import shlex
from xml.sax.saxutils import escape
from pathlib import Path

source, target, binary, codex_home, log_dir, env_file = sys.argv[1:]
body = Path(source).read_text()
body = body.replace("__CODEX_HOME__", escape(codex_home))
body = body.replace("__LOG_DIR__", escape(log_dir))
command = f"set -a; . {shlex.quote(env_file)}; exec {shlex.quote(binary)} telegram"
body = body.replace("__COMMAND__", escape(command))
Path(target).write_text(body)
PY
    plutil -lint "$target" >/dev/null
    printf 'Installed launchd agent at %s\n' "$target"
    printf 'Enable it with:\n  launchctl bootstrap gui/%s %s\n' "$(id -u)" "$target"
}

run_health_check() {
    local pfterminal_bin
    pfterminal_bin="$(command -v pfterminal || true)"
    [[ "$pfterminal_bin" == /* ]] || die "pfterminal was not found at an absolute PATH entry"
    printf 'Running Telegram health check before service installation...\n'
    set -a
    # This file is created mode 0600 by this script and is the same input the
    # managed service will consume.
    # shellcheck disable=SC1090
    source "$ENV_FILE"
    set +a
    CODEX_HOME="$CODEX_HOME_RESOLVED" "$pfterminal_bin" telegram --health
}

TOKEN_VALUE=""
WORKSPACE="$DEFAULT_WORKSPACE"
ENV_FILE="$DEFAULT_ENV_FILE"
APPROVAL_POLICY="on-request"
INSTALL_SYSTEMD=0
INSTALL_LAUNCHD=0
NO_TOKEN_REQUIRED=0
ALLOW_DANGER_FULL_ACCESS=0
CHAT_IDS_EXPLICIT=0
USER_IDS_EXPLICIT=0
WORKSPACE_EXPLICIT=0
APPROVAL_POLICY_EXPLICIT=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --token|--token=*) die "do not pass Telegram bot tokens on the command line; set $TOKEN_ENV_VAR or enter the prompt" ;;
        --chat-id) [[ $# -ge 2 ]] || die "--chat-id requires a value"; add_chat_ids "$2"; CHAT_IDS_EXPLICIT=1; shift 2 ;;
        --chat-id=*) add_chat_ids "${1#--chat-id=}"; CHAT_IDS_EXPLICIT=1; shift ;;
        --user-id) [[ $# -ge 2 ]] || die "--user-id requires a value"; add_user_ids "$2"; USER_IDS_EXPLICIT=1; shift 2 ;;
        --user-id=*) add_user_ids "${1#--user-id=}"; USER_IDS_EXPLICIT=1; shift ;;
        --workspace) [[ $# -ge 2 ]] || die "--workspace requires a value"; WORKSPACE=$2; WORKSPACE_EXPLICIT=1; shift 2 ;;
        --workspace=*) WORKSPACE=${1#--workspace=}; WORKSPACE_EXPLICIT=1; shift ;;
        --env-file) [[ $# -ge 2 ]] || die "--env-file requires a value"; ENV_FILE=$2; shift 2 ;;
        --env-file=*) ENV_FILE=${1#--env-file=}; shift ;;
        --approval-policy) [[ $# -ge 2 ]] || die "--approval-policy requires a value"; APPROVAL_POLICY=$2; APPROVAL_POLICY_EXPLICIT=1; shift 2 ;;
        --approval-policy=*) APPROVAL_POLICY=${1#--approval-policy=}; APPROVAL_POLICY_EXPLICIT=1; shift ;;
        --allow-danger-full-access) ALLOW_DANGER_FULL_ACCESS=1; shift ;;
        --install-systemd) INSTALL_SYSTEMD=1; shift ;;
        --install-launchd) INSTALL_LAUNCHD=1; shift ;;
        --no-token-required) NO_TOKEN_REQUIRED=1; shift ;;
        -h|--help) usage; exit 0 ;;
        --) shift; break ;;
        -*) die "unknown option: $1" ;;
        *) die "unexpected argument: $1" ;;
    esac
done

[[ $# -eq 0 ]] || die "unexpected argument: $1"
need_python

CODEX_HOME_RESOLVED="$(resolve_codex_home)"
CONFIG_PATH="$CODEX_HOME_RESOLVED/config.toml"
WORKSPACE="$(abs_dir "$WORKSPACE")"
ENV_FILE="$(abs_file "$ENV_FILE")"

if [[ ${#CHAT_IDS[@]} -eq 0 ]]; then
    EXISTING_CHAT_IDS="$(read_existing_ids "$CONFIG_PATH" allowed_chat_ids '-?\d+')"
    if [[ -n "$EXISTING_CHAT_IDS" ]]; then
        add_chat_ids "$EXISTING_CHAT_IDS"
        printf 'Using existing Telegram allowed_chat_ids from %s\n' "$CONFIG_PATH"
    elif [[ -t 0 ]]; then
        read -r -p "Telegram chat ID(s), comma-separated: " CHAT_ID_INPUT
        add_chat_ids "$CHAT_ID_INPUT"
        CHAT_IDS_EXPLICIT=1
    else
        die "provide at least one --chat-id in non-interactive mode"
    fi
fi
[[ ${#CHAT_IDS[@]} -gt 0 ]] || die "at least one Telegram chat ID is required"
if [[ ${#USER_IDS[@]} -eq 0 ]]; then
    EXISTING_USER_IDS="$(read_existing_ids "$CONFIG_PATH" allowed_user_ids '\d+')"
    if [[ -n "$EXISTING_USER_IDS" ]]; then
        add_user_ids "$EXISTING_USER_IDS"
        printf 'Using existing Telegram allowed_user_ids from %s\n' "$CONFIG_PATH"
    fi
fi
if printf '%s\n' "${CHAT_IDS[@]}" | grep -q '^-'; then
    if [[ ${#USER_IDS[@]} -eq 0 && -t 0 ]]; then
        read -r -p "Telegram user ID(s) allowed to act in groups, comma-separated: " USER_ID_INPUT
        add_user_ids "$USER_ID_INPUT"
        USER_IDS_EXPLICIT=1
    fi
    [[ ${#USER_IDS[@]} -gt 0 ]] || die "group chat IDs require at least one --user-id"
fi

if [[ $NO_TOKEN_REQUIRED -eq 0 && -z "$TOKEN_VALUE" ]]; then
    if [[ -n "${!TOKEN_ENV_VAR:-}" ]]; then
        TOKEN_VALUE=${!TOKEN_ENV_VAR}
    elif env_file_has_token "$ENV_FILE"; then
        printf 'Using existing %s entry in %s\n' "$TOKEN_ENV_VAR" "$ENV_FILE"
    elif [[ -t 0 ]]; then
        read -r -s -p "Telegram bot token: " TOKEN_VALUE
        printf '\n'
    else
        die "set $TOKEN_ENV_VAR, use an existing env file entry, or use --no-token-required"
    fi
fi

SANDBOX_ISSUE="$(sandbox_issue)"
SANDBOX_MODE_TO_SET=""
if [[ -n "$SANDBOX_ISSUE" ]]; then
    printf 'Sandbox preflight: %s.\n' "$SANDBOX_ISSUE"
    printf 'Remediation: install bwrap and enable unprivileged user namespaces, then rerun this script.\n'
    printf 'On Ubuntu 24.04, also ensure kernel.apparmor_restrict_unprivileged_userns allows bwrap to start.\n'
    EXISTING_SANDBOX_MODE="$(read_existing_sandbox_mode "$CONFIG_PATH")"
    if [[ "$EXISTING_SANDBOX_MODE" == "danger-full-access" ]]; then
        printf 'Existing config already sets sandbox_mode = "danger-full-access"; leaving it unchanged.\n'
    elif [[ $ALLOW_DANGER_FULL_ACCESS -eq 1 ]]; then
        SANDBOX_MODE_TO_SET="danger-full-access"
        printf 'Writing top-level sandbox_mode = "danger-full-access" because --allow-danger-full-access was passed.\n'
    elif [[ -t 0 ]]; then
        read -r -p 'Set sandbox_mode = "danger-full-access" globally? This disables the sandbox for all PFTerminal surfaces and is only appropriate on a trusted single-user host. Type y to continue [y/N]: ' DANGER_REPLY
        if [[ "$DANGER_REPLY" == "y" || "$DANGER_REPLY" == "Y" ]]; then
            SANDBOX_MODE_TO_SET="danger-full-access"
            printf 'Writing top-level sandbox_mode = "danger-full-access" after interactive confirmation.\n'
        else
            die 'not writing sandbox_mode; fix the sandbox preflight issue or rerun with --allow-danger-full-access'
        fi
    else
        die 'not writing sandbox_mode in non-interactive mode; fix the sandbox preflight issue or pass --allow-danger-full-access'
    fi
else
    printf 'Sandbox preflight: OK; leaving existing sandbox_mode/default unchanged.\n'
fi

write_env_file "$ENV_FILE" "$CODEX_HOME_RESOLVED" "$TOKEN_VALUE"
printf 'Wrote environment file at %s\n' "$ENV_FILE"
backup_config "$CONFIG_PATH"
EXPLICIT_CONFIG_KEYS=""
[[ $CHAT_IDS_EXPLICIT -eq 1 ]] && EXPLICIT_CONFIG_KEYS="${EXPLICIT_CONFIG_KEYS},allowed_chat_ids"
[[ $USER_IDS_EXPLICIT -eq 1 ]] && EXPLICIT_CONFIG_KEYS="${EXPLICIT_CONFIG_KEYS},allowed_user_ids"
[[ $WORKSPACE_EXPLICIT -eq 1 ]] && EXPLICIT_CONFIG_KEYS="${EXPLICIT_CONFIG_KEYS},default_cwd"
[[ $APPROVAL_POLICY_EXPLICIT -eq 1 ]] && EXPLICIT_CONFIG_KEYS="${EXPLICIT_CONFIG_KEYS},approval_policy"
EXPLICIT_CONFIG_KEYS="${EXPLICIT_CONFIG_KEYS#,}"
CHAT_IDS_CSV="$(IFS=,; printf '%s' "${CHAT_IDS[*]}")"
USER_IDS_CSV="$(IFS=,; printf '%s' "${USER_IDS[*]}")"
merge_config "$CONFIG_PATH" "$WORKSPACE" "$SANDBOX_MODE_TO_SET" "$APPROVAL_POLICY" "$EXPLICIT_CONFIG_KEYS" "$CHAT_IDS_CSV" "$USER_IDS_CSV"
printf 'Configured Telegram connector in %s\n' "$CONFIG_PATH"
seed_agents_md "$WORKSPACE"
[[ $NO_TOKEN_REQUIRED -eq 1 ]] || run_health_check
[[ $INSTALL_SYSTEMD -eq 1 ]] && install_systemd_unit
[[ $INSTALL_LAUNCHD -eq 1 ]] && install_launchd_unit

printf '\nNext steps:\n'
printf '  CODEX_HOME=%s pfterminal telegram\n' "$CODEX_HOME_RESOLVED"
printf '  For systemd, ensure %s contains %s, then run the enable/start commands printed by --install-systemd.\n' "$ENV_FILE" "$TOKEN_ENV_VAR"
printf '  Send a Telegram message from an allowed chat ID to test the connector.\n'
