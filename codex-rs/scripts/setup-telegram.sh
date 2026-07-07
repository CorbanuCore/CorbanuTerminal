#!/usr/bin/env bash
set -euo pipefail

: "${HOME:?HOME is required}"

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
CODEX_RS_DIR="$(cd -- "$SCRIPT_DIR/.." && pwd -P)"
TOKEN_ENV_VAR="PFTERMINAL_TELEGRAM_TOKEN"
DEFAULT_ENV_FILE="$HOME/.config/pfterminal/telegram.env"

usage() {
    cat <<'USAGE'
Usage: setup-telegram.sh [OPTIONS] [BOT_TOKEN]

Options:
  --token TOKEN          Telegram bot token; also accepted as the first positional arg.
  --chat-id ID          Allowed chat ID. Repeat or pass comma-separated IDs.
  --workspace DIR       Telegram default_cwd. Defaults to $HOME.
  --env-file PATH       EnvironmentFile to write. Defaults to ~/.config/pfterminal/telegram.env.
  --approval-policy VAL Approval policy. Defaults to on-request.
  --install-systemd     Install the systemd --user service template.
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

read_existing_chat_ids() {
    [[ -f "$1" ]] || return 0
    python3 - "$1" <<'PY'
import ast
import sys
from pathlib import Path

section = None
for raw in Path(sys.argv[1]).read_text().splitlines():
    line = raw.strip()
    if not line or line.startswith("#"):
        continue
    if line.startswith("[") and line.endswith("]"):
        section = line.strip("[]")
        continue
    if section == "telegram" and line.startswith("allowed_chat_ids") and "=" in line:
        try:
            ids = ast.literal_eval(line.split("=", 1)[1].strip())
        except Exception as exc:
            print(f"failed to parse existing allowed_chat_ids: {exc}", file=sys.stderr)
            sys.exit(1)
        if ids:
            print(",".join(str(int(item)) for item in ids))
        break
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
    if [[ -r /proc/sys/kernel/unprivileged_userns_clone ]]; then
        value="$(tr -d '[:space:]' < /proc/sys/kernel/unprivileged_userns_clone)"
        if [[ "$value" == "0" ]]; then
            printf '/proc/sys/kernel/unprivileged_userns_clone reads as 0'
            return 0
        fi
    fi
}

env_file_has_token() {
    [[ -f "$1" ]] || return 1
    grep -Eq "^${TOKEN_ENV_VAR}=.+" "$1"
}

write_env_file() {
    local token_action=preserve
    [[ -n "$3" ]] && token_action=set
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
path.parent.mkdir(parents=True, exist_ok=True)
path.write_text("\n".join(out) + "\n")
PY
    chmod 600 -- "$1"
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
    shift 4
    python3 - "$config_path" "$workspace" "$sandbox_mode" "$approval_policy" "$@" <<'PY'
import json
import re
import sys
from pathlib import Path

path = Path(sys.argv[1])
updates = {
    "enabled": True,
    "bot_token_env": "PFTERMINAL_TELEGRAM_TOKEN",
    "allowed_chat_ids": [int(value) for value in sys.argv[5:]],
    "mode": "polling",
    "approval_policy": sys.argv[4],
    "default_cwd": sys.argv[2],
}
order = list(updates)
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
    missing = [key for key in key_order if key not in seen]
    out.extend(f"{key} = {fmt(values[key])}" for key in missing)
    return out, missing

def table_range(lines, name):
    for start, line in enumerate(lines):
        if section(line) == name:
            end = next((i for i in range(start + 1, len(lines)) if section(lines[i]) is not None), len(lines))
            return start, end
    return None

lines = (path.read_text() if path.exists() else "").splitlines()
if top:
    first_table = next((i for i, line in enumerate(lines) if section(line) is not None), len(lines))
    head, missing = apply_range(lines, 0, first_table, top, list(top))
    if missing and first_table < len(lines):
        head.append("")
    lines = head + lines[first_table:]

table = table_range(lines, "telegram")
if table is None:
    if lines and lines[-1].strip():
        lines.append("")
    lines.append("[telegram]")
    lines.extend(f"{key} = {fmt(updates[key])}" for key in order)
else:
    start, end = table
    body, _missing = apply_range(lines, start + 1, end, updates, order)
    lines = lines[: start + 1] + body + lines[end:]

output = "\n".join(lines) + "\n"
try:
    import tomllib
    tomllib.loads(output)
except ModuleNotFoundError:
    pass
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
    [[ -r "$source" ]] || die "missing systemd service template: $source"
    mkdir -p -- "$target_dir"
    cp -f -- "$source" "$target_dir/pfterminal-telegram.service"
    printf 'Installed systemd user unit at %s\n' "$target_dir/pfterminal-telegram.service"
    printf 'Enable it with:\n  systemctl --user daemon-reload\n  systemctl --user enable --now pfterminal-telegram.service\n'
}

TOKEN_VALUE=""
TOKEN_FROM_ARG=0
WORKSPACE="$HOME"
ENV_FILE="$DEFAULT_ENV_FILE"
APPROVAL_POLICY="on-request"
INSTALL_SYSTEMD=0
NO_TOKEN_REQUIRED=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --token) [[ $# -ge 2 ]] || die "--token requires a value"; TOKEN_VALUE=$2; TOKEN_FROM_ARG=1; shift 2 ;;
        --token=*) TOKEN_VALUE=${1#--token=}; TOKEN_FROM_ARG=1; shift ;;
        --chat-id) [[ $# -ge 2 ]] || die "--chat-id requires a value"; add_chat_ids "$2"; shift 2 ;;
        --chat-id=*) add_chat_ids "${1#--chat-id=}"; shift ;;
        --workspace) [[ $# -ge 2 ]] || die "--workspace requires a value"; WORKSPACE=$2; shift 2 ;;
        --workspace=*) WORKSPACE=${1#--workspace=}; shift ;;
        --env-file) [[ $# -ge 2 ]] || die "--env-file requires a value"; ENV_FILE=$2; shift 2 ;;
        --env-file=*) ENV_FILE=${1#--env-file=}; shift ;;
        --approval-policy) [[ $# -ge 2 ]] || die "--approval-policy requires a value"; APPROVAL_POLICY=$2; shift 2 ;;
        --approval-policy=*) APPROVAL_POLICY=${1#--approval-policy=}; shift ;;
        --install-systemd) INSTALL_SYSTEMD=1; shift ;;
        --no-token-required) NO_TOKEN_REQUIRED=1; shift ;;
        -h|--help) usage; exit 0 ;;
        --) shift; break ;;
        -*) die "unknown option: $1" ;;
        *)
            if [[ $TOKEN_FROM_ARG -eq 0 && -z "$TOKEN_VALUE" ]]; then
                TOKEN_VALUE=$1
                TOKEN_FROM_ARG=1
                shift
            else
                die "unexpected argument: $1"
            fi
            ;;
    esac
done

[[ $# -eq 0 ]] || die "unexpected argument: $1"
need_python

CODEX_HOME_RESOLVED="$(resolve_codex_home)"
CONFIG_PATH="$CODEX_HOME_RESOLVED/config.toml"
WORKSPACE="$(abs_dir "$WORKSPACE")"
ENV_FILE="$(abs_file "$ENV_FILE")"

if [[ ${#CHAT_IDS[@]} -eq 0 ]]; then
    EXISTING_CHAT_IDS="$(read_existing_chat_ids "$CONFIG_PATH")"
    if [[ -n "$EXISTING_CHAT_IDS" ]]; then
        add_chat_ids "$EXISTING_CHAT_IDS"
        printf 'Using existing Telegram allowed_chat_ids from %s\n' "$CONFIG_PATH"
    elif [[ -t 0 ]]; then
        read -r -p "Telegram chat ID(s), comma-separated: " CHAT_ID_INPUT
        add_chat_ids "$CHAT_ID_INPUT"
    else
        die "provide at least one --chat-id in non-interactive mode"
    fi
fi
[[ ${#CHAT_IDS[@]} -gt 0 ]] || die "at least one Telegram chat ID is required"

if [[ $NO_TOKEN_REQUIRED -eq 0 && -z "$TOKEN_VALUE" ]]; then
    if [[ -n "${!TOKEN_ENV_VAR:-}" ]]; then
        TOKEN_VALUE=${!TOKEN_ENV_VAR}
    elif env_file_has_token "$ENV_FILE"; then
        printf 'Using existing %s entry in %s\n' "$TOKEN_ENV_VAR" "$ENV_FILE"
    elif [[ -t 0 ]]; then
        read -r -s -p "Telegram bot token: " TOKEN_VALUE
        printf '\n'
    else
        die "provide --token, set $TOKEN_ENV_VAR, or use --no-token-required"
    fi
fi

SANDBOX_ISSUE="$(sandbox_issue)"
SANDBOX_MODE_TO_SET=""
if [[ -n "$SANDBOX_ISSUE" ]]; then
    SANDBOX_MODE_TO_SET="danger-full-access"
    printf 'Sandbox preflight: %s.\n' "$SANDBOX_ISSUE"
    printf 'Setting sandbox_mode = "danger-full-access" for this trusted-host always-on setup; otherwise enable unprivileged user namespaces and bwrap.\n'
else
    printf 'Sandbox preflight: OK; leaving existing sandbox_mode/default unchanged.\n'
fi

write_env_file "$ENV_FILE" "$CODEX_HOME_RESOLVED" "$TOKEN_VALUE"
printf 'Wrote environment file at %s\n' "$ENV_FILE"
backup_config "$CONFIG_PATH"
merge_config "$CONFIG_PATH" "$WORKSPACE" "$SANDBOX_MODE_TO_SET" "$APPROVAL_POLICY" "${CHAT_IDS[@]}"
printf 'Configured Telegram connector in %s\n' "$CONFIG_PATH"
seed_agents_md "$WORKSPACE"
[[ $INSTALL_SYSTEMD -eq 1 ]] && install_systemd_unit

printf '\nNext steps:\n'
printf '  CODEX_HOME=%s pfterminal telegram\n' "$CODEX_HOME_RESOLVED"
printf '  For systemd, ensure %s contains %s, then run the enable/start commands printed by --install-systemd.\n' "$ENV_FILE" "$TOKEN_ENV_VAR"
printf '  Send a Telegram message from an allowed chat ID to test the connector.\n'
