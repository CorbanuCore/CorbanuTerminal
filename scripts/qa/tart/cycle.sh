#!/bin/bash
# Measure clone -> boot -> SSH -> copy the signed binary in -> one model turn -> stop -> delete.
#
#   TART_QA_WORK=... scripts/qa/tart/cycle.sh [RUNS]
#
# Needs the base from prepare-base.sh. No shared folder: the binary goes in over SSH. The model key
# is read from the Corbanu vault at use time, sent over the SSH session's stdin into the guest
# command's environment, and never written to the guest disk, the host disk or a log.
# Env: CORBANU_BIN (default: the binary behind ~/.local/bin/corbanu), TART_QA_ZAI_LABEL (vault label,
# default provider/zai_api_key), TART_QA_MODEL (default glm-5.3-flash).
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=SCRIPTDIR/lib.sh
. "$here/lib.sh"

RUNS="${1:-3}"
MODEL="${TART_QA_MODEL:-glm-5.3-flash}"
LABEL="${TART_QA_ZAI_LABEL:-provider/zai_api_key}"
RESULTS="$LOGS/cycle-results.jsonl"

resolve_bin() {
  local launcher="$HOME/.local/bin/corbanu" work
  if [ -n "${CORBANU_BIN:-}" ]; then echo "$CORBANU_BIN"; return; fi
  if file "$launcher" | grep -q 'shell script'; then
    # shellcheck disable=SC2016  # a literal ${...} in the launcher
    work="$(sed -n 's/^CORBANU_WORK_DIR="\${CORBANU_LAUNCHER_WORK_DIR:-\(.*\)}"$/\1/p' "$launcher")"
    python3 -c 'import os,sys; print(os.path.realpath(sys.argv[1]))' "$work/bin/corbanu"
  else
    python3 -c 'import os,sys; print(os.path.realpath(sys.argv[1]))' "$launcher"
  fi
}
SRC_BIN="$(resolve_bin)"
# Work from a private copy: a reinstall on the host mid-run must not change the binary under test.
BIN="$LOGS/corbanu-under-test"
rm -f "$BIN"; cp -c "$SRC_BIN" "$BIN" 2> /dev/null || cp "$SRC_BIN" "$BIN"
codesign --verify --strict "$BIN" || die "$SRC_BIN is not validly signed"
BIN_SHA="$(shasum -a 256 "$BIN" | cut -d' ' -f1)"
log "binary $SRC_BIN sha256 $BIN_SHA"

# The vault helper must see the person's default home, not a disposable one this shell may carry.
zai_key() {
  /usr/bin/env -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME -u CORBANU_TEST_NO_NATIVE_KEYRING \
    HOME="$HOME" "$HOME/.local/bin/corbanu" vault auth-helper "$LABEL"
}

# Bytes the clone holds on its own (APFS clone cost), in MB.
private_mb() { echo $(($(python3 "$here/apfs_private_size.py" "$TART_HOME/vms/$1") / 1048576)); }

sample_cpu() { # PID OUT: one line per 2 s: %cpu rss_kb
  while kill -0 "$1" 2> /dev/null; do ps -o %cpu= -o rss= -p "$1" >> "$2" 2> /dev/null || true; sleep 2; done
}

one_cycle() {
  local i="$1" name ip pid before t0 t_clone t_ip t_ssh t_copy t_smoke t_stop t_del priv_clone priv_run
  local smoke_ok guest_sha guest_os rss_ssh rss_smoke
  name="qa-cycle-$(date -u +%Y%m%dT%H%M%SZ)-$i"
  CURRENT="$name"
  t0="$(now)"
  "$TART" clone "$TART_QA_BASE" "$name"
  t_clone="$(now)"
  priv_clone="$(private_mb "$name")"
  before="$(vm_pids)"
  start_vm "$name"
  ip="$(wait_ip "$name" 180)"
  t_ip="$(now)"
  wait_ssh "$ip" 180 || die "no SSH on $ip"
  t_ssh="$(now)"
  pid="$(comm -13 <(echo "$before") <(vm_pids) | head -1)"
  rss_ssh="$(ps -o rss= -p "$pid" | tr -d ' ')"
  sample_cpu "$pid" "$LOGS/$name.cpu" &

  gssh "$ip" 'mkdir -p ~/qa/bin && cat > ~/qa/bin/corbanu && chmod 755 ~/qa/bin/corbanu' < "$BIN"
  guest_sha="$(gssh "$ip" 'codesign --verify --strict ~/qa/bin/corbanu && shasum -a 256 ~/qa/bin/corbanu' | cut -d' ' -f1)"
  [ "$guest_sha" = "$BIN_SHA" ] || die "binary hash differs in the guest"
  t_copy="$(now)"

  # One retry pair for a transient vault or network failure; the attempt count is recorded.
  local attempt
  for attempt in 1 2 3; do
    # shellcheck disable=SC2016  # expanded in the guest
    printf '%s\n' "$(zai_key)" | gssh "$ip" 'IFS= read -r ZAI_API_KEY; export ZAI_API_KEY
      h="$(mktemp -d /tmp/qa-smoke.XXXXXX)"
      CORBANU_HOME="$h" CODEX_HOME="$h" CORBANU_TEST_NO_NATIVE_KEYRING=1 ~/qa/bin/corbanu exec --json \
        --skip-git-repo-check --ephemeral -m '"$MODEL"' -c model_provider=\"zai\" \
        -c cli_auth_credentials_store=\"file\" -C "$h" "Reply with exactly: SMOKE-OK" < /dev/null
      rc=$?; rm -r "$h"; exit $rc' > "$LOGS/$name.smoke.jsonl" 2> "$LOGS/$name.smoke.stderr" && break
    log "smoke attempt $attempt failed"; sleep 3
  done
  t_smoke="$(now)"
  smoke_ok="$(python3 - "$LOGS/$name.smoke.jsonl" << 'PY'
import json, sys
texts = []
for line in open(sys.argv[1]):
    try:
        ev = json.loads(line)
    except ValueError:
        continue
    item = ev.get("item") or {}
    if ev.get("type") == "item.completed" and item.get("type") == "agent_message":
        texts.append(item.get("text", "").strip())
print("true" if texts and texts[-1] == "SMOKE-OK" else "false")
PY
)"
  rss_smoke="$(ps -o rss= -p "$pid" | tr -d ' ')"
  guest_os="$(gssh "$ip" 'sw_vers -productVersion')"
  stop_vm "$name" 30
  t_stop="$(now)"
  priv_run="$(private_mb "$name")"
  "$TART" delete "$name"
  CURRENT=""
  t_del="$(now)"
  RUN="$name" OS="$guest_os" ATTEMPTS="$attempt" SHA="$BIN_SHA" OK="$smoke_ok" \
    T="$t0 $t_clone $t_ip $t_ssh $t_copy $t_smoke $t_stop $t_del" RSS="$rss_ssh $rss_smoke" \
    PRIV="$priv_clone $priv_run" python3 - "$RESULTS" "$LOGS/$name.cpu" "$MODEL" << 'PY'
import json, os, statistics, sys
t0, clone, ip, ssh, copy, smoke, stop, delete = map(float, os.environ["T"].split())
rss_ssh, rss_smoke = map(int, os.environ["RSS"].split())
priv_clone, priv_run = map(int, os.environ["PRIV"].split())
cpu = [float(line.split()[0]) for line in open(sys.argv[2]) if line.strip()]
row = dict(run=os.environ["RUN"], guest_os=os.environ["OS"], binary_sha256=os.environ["SHA"], model=sys.argv[3],
           smoke_ok=os.environ["OK"] == "true", smoke_attempts=int(os.environ["ATTEMPTS"]),
           clone_s=round(clone - t0, 1), boot_to_ip_s=round(ip - clone, 1), ip_to_ssh_s=round(ssh - ip, 1),
           clone_to_ssh_s=round(ssh - t0, 1), copy_verify_s=round(copy - ssh, 1), smoke_s=round(smoke - copy, 1),
           stop_s=round(stop - smoke, 1), delete_s=round(delete - stop, 1), total_s=round(delete - t0, 1),
           rss_mb_at_ssh=round(rss_ssh / 1024), rss_mb_after_smoke=round(rss_smoke / 1024),
           cpu_pct_mean=round(statistics.mean(cpu), 1) if cpu else None, cpu_pct_max=max(cpu) if cpu else None,
           clone_private_mb_after_clone=priv_clone, clone_private_mb_after_run=priv_run)
with open(sys.argv[1], "a") as out:
    out.write(json.dumps(row) + "\n")
print(json.dumps(row))
PY
}

CURRENT=""
cleanup() { if [ -n "$CURRENT" ]; then stop_vm "$CURRENT" 5; "$TART" delete "$CURRENT" || true; fi; }
trap cleanup EXIT
for i in $(seq 1 "$RUNS"); do one_cycle "$i"; done
python3 - "$RESULTS" "$RUNS" << 'PY'
import json, statistics, sys
rows = [json.loads(l) for l in open(sys.argv[1])][-int(sys.argv[2]):]
for key in ("clone_s", "boot_to_ip_s", "ip_to_ssh_s", "clone_to_ssh_s", "copy_verify_s", "smoke_s", "stop_s", "delete_s", "total_s"):
    print(f"{key:16} median {statistics.median(r[key] for r in rows):7.1f}  runs {[r[key] for r in rows]}")
print("smoke_ok", [r["smoke_ok"] for r in rows])
PY
