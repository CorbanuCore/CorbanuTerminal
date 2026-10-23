# shellcheck shell=bash
# Shared settings and helpers for the Tart QA scripts. Source it; do not run it.
#
# Everything lives under TART_QA_WORK (VM images, keys, logs). Point it at a disk with room:
# the base image takes about 25-30 GB; a clone costs little until its guest writes.

: "${TART_QA_WORK:?set TART_QA_WORK to a work directory on a disk with at least 60 GB free}"
export TART_HOME="${TART_HOME:-$TART_QA_WORK/tart-home}"
export TMPDIR="$TART_HOME/tmp/"
# Tart otherwise prunes its caches on its own when the disk is tight; keep every deletion explicit.
export TART_NO_AUTO_PRUNE=1
TART="${TART:-tart}"
TART_QA_BASE="${TART_QA_BASE:-corbanu-qa-base}"
TART_QA_KEY="${TART_QA_KEY:-$TART_QA_WORK/keys/tart-qa_ed25519}"
TART_QA_KNOWN_HOSTS="${TART_QA_KNOWN_HOSTS:-$TART_QA_WORK/keys/known_hosts}"
UTMCTL="${UTMCTL:-/Applications/UTM.app/Contents/MacOS/utmctl}"
LOGS="$TART_QA_WORK/logs"
mkdir -p "$TMPDIR" "$LOGS"

log() { printf '%s %s\n' "$(date -u +%FT%TZ)" "$*" >&2; }
die() { log "ERROR: $*"; exit 1; }
now() { python3 -c 'import time; print(f"{time.time():.3f}")'; }

# Running macOS guests on this host. macOS allows two at once, shared by every tool (UTM, Tart, ...).
count_procs() { # ERE -> number of matching processes; exits non-zero with no number if pgrep errors
  local out rc=0
  out="$(pgrep -f "$1")" || rc=$?
  [ "$rc" -le 1 ] || die "pgrep failed ($rc)"
  if [ -n "$out" ]; then grep -c . <<< "$out"; else echo 0; fi
}
# Tart VMs running on the host: `tart run` processes however tart was started (bundle path, PATH,
# Homebrew wrapper; any TART_HOME), or this TART_HOME's own list if that says more. Prints "?" when
# the process count is unknown, which guard_slot treats as "running".
running_tart_vms() {
  local procs listed
  procs="$(count_procs '(^|/)tart run( |$)')" || procs=""
  listed="$("$TART" list --source local --format json 2> /dev/null |
    python3 -c 'import json,sys; print(sum(1 for v in json.load(sys.stdin) if v.get("Running") or v.get("State") == "running"))' 2> /dev/null)" ||
    { log "tart list failed; using the process count only"; listed=0; }
  case "$procs" in '' | *[!0-9]*) echo "?"; return 0 ;; esac
  echo $((procs > listed ? procs : listed))
}
running_utm_vms() { # informational only (guard_slot counts VM processes); never launches UTM
  local out
  pgrep -x UTM > /dev/null || { echo 0; return 0; }
  out="$("$UTMCTL" list 2> /dev/null)" || { echo "?"; return 0; }
  awk 'NR > 1 && $2 == "started"' <<< "$out" | grep -c . || true
}

# Refuse to start a VM unless it would be the only Tart VM and a macOS slot is free. Never touches other
# VMs. The total counts every Virtualization.framework guest process on the host (UTM's, and Tart VMs of
# any TART_HOME), so a failing `utmctl` or another Tart home can't hide a running guest.
guard_slot() {
  local tart_running total
  tart_running="$(running_tart_vms)"
  total="$(count_procs com.apple.Virtualization.VirtualMachine)"
  [ "$tart_running" = 0 ] || die "a Tart VM is already running; these scripts run one at a time"
  [ "$total" -lt 2 ] || die "$total guests already running (UTM: $(running_utm_vms)); no free macOS slot"
  log "slot ok (guests running: $total; UTM: $(running_utm_vms); Tart: $tart_running)"
}

SSH_OPTS=(-i "$TART_QA_KEY" -o IdentitiesOnly=yes -o UserKnownHostsFile="$TART_QA_KNOWN_HOSTS"
  -o StrictHostKeyChecking=yes -o HostKeyAlias=tart-qa-base -o BatchMode=yes -o ConnectTimeout=5
  -o ForwardAgent=no -o ClearAllForwardings=yes -o LogLevel=ERROR)
# Every clone of the base shares its SSH host key, which prepare-base.sh pins under the alias above.
# shellcheck disable=SC2029  # remote commands are built on purpose
gssh() { local ip="$1"; shift; ssh "${SSH_OPTS[@]}" "admin@$ip" "$@"; }

start_vm() { # NAME: run the VM headless in the background, without host shares, audio or clipboard
  local name="$1"
  guard_slot
  nohup "$TART" run --no-graphics --no-audio --no-clipboard "$name" > "$LOGS/$name.run.log" 2>&1 &
  echo $! > "$LOGS/$name.pid"
}

wait_ip() { "$TART" ip --wait "${2:-180}" "$1"; }

wait_ssh() { # IP [SECONDS]
  local ip="$1" limit="${2:-180}" start
  start=$(date +%s)
  until gssh "$ip" true 2>/dev/null; do
    [ $(($(date +%s) - start)) -lt "$limit" ] || return 1
    sleep 1
  done
}

stop_vm() { # NAME [TIMEOUT]: shutdown request, power-off after TIMEOUT seconds; waits for `tart run` to exit
  local name="$1" pid
  "$TART" stop --timeout "${2:-30}" "$name" > /dev/null 2>&1 || true
  if [ -f "$LOGS/$name.pid" ]; then
    pid="$(cat "$LOGS/$name.pid")"
    for _ in $(seq 1 120); do kill -0 "$pid" 2> /dev/null || break; sleep 0.5; done
    kill -0 "$pid" 2> /dev/null && { log "tart run $pid still up after 60 s; killing it"; kill -9 "$pid" 2> /dev/null || true; }
    rm "$LOGS/$name.pid"
  fi
}

# PIDs of Virtualization.framework VM processes (one per running guest, UTM's included); empty if none.
vm_pids() { { pgrep -f com.apple.Virtualization.VirtualMachine || [ $? -eq 1 ]; } | sort; }
