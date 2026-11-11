#!/bin/bash
# utm-par2 host-side driver (owner decision D93): the Tart parity run's harness (tart-par1 h/, unchanged guest files)
# on the long-lived UTM guest "macOS 3", with the same build 8c73c8ea80 and the same packet. Only the VM layer differs
# from tart-run.sh: the guest is started and stopped with utmctl instead of cloned and deleted; root actions go through
# the UTM harness's admin helper (su.exp, password from the vault at use time); the guest's own seal (pf anchor, pinned
# api.z.ai, coordinator account) replaces seal-guest.sh; the campaign is provisioned once and groups run one at a time
# on rotating lanes with a lane wipe before and after each (the UTM qual method), then restore.sh reverses provisioning.
#   utm-run.sh up | prep | tunnel | stage | gate | group LANE GROUP [LABEL] | status | down | verify-boot
# "macOS 2" is never touched. No real key is placed in the guest: the host relay (127.0.0.1:11237) adds the Z.AI key
# and the guest reaches it only through one SSH reverse tunnel on its loopback (18601), as on Tart.
set -euo pipefail
Q=/Volumes/CorbanuDrive/Corbanu/.codex-work/utm-par2-20261021
WORKERS=/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002
U=/Applications/UTM.app/Contents/MacOS/utmctl
VM="macOS 3"
G=agent@192.168.64.3
LOCK=$WORKERS/vm-macos3.lock
TLOCK=$WORKERS/vm-tart.lock
TM=(/opt/homebrew/bin/tmux -S "$Q/tmux.sock")
C=$Q/${PAR_COMMON:?set PAR_COMMON to the evidence folder, e.g. common/utm}
KEY=/Volumes/CorbanuDrive/Corbanu/.codex-work/functional-pf83.20260915/keys/pf83-vm_ed25519
KH=/Volumes/CorbanuDrive/Corbanu/.codex-work/aggressive-rerun-20261011/known_hosts
SSH_OPTS=(-i "$KEY" -o IdentitiesOnly=yes -o StrictHostKeyChecking=yes -o UserKnownHostsFile="$KH" -o ConnectTimeout=10
  -o BatchMode=yes -o ForwardAgent=no -o ClearAllForwardings=yes)
TUN_OPTS=(-i "$KEY" -o IdentitiesOnly=yes -o StrictHostKeyChecking=yes -o UserKnownHostsFile="$KH" -o ConnectTimeout=5
  -o BatchMode=yes -o ForwardAgent=no -o ExitOnForwardFailure=yes -o ServerAliveInterval=15 -o ServerAliveCountMax=4
  -o LogLevel=ERROR -N -R 127.0.0.1:18601:127.0.0.1:11237)
die() { echo "utm-run: $*" >&2; exit 1; }
ag() { ssh -n "${SSH_OPTS[@]}" "$G" "$@"; }
agi() { ssh "${SSH_OPTS[@]}" "$G" "$@"; }
# root SCRIPT: runs a local script as root in the guest (admin helper; the admin password comes from the vault only here).
root() {
  local name="agadmin-$(date +%s)-$$.sh"
  agi "mkdir -p -m 700 ~/ag-admin && cat > ~/ag-admin/$name && chmod 755 ~/ag-admin ~/ag-admin/$name" < "$1"
  agi "cat > ~/ag-admin/su.exp" < "$Q/hu/su.exp"
  NEOPW="$(/usr/bin/env -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME -u CORBANU_TEST_NO_NATIVE_KEYRING HOME="$HOME" "$HOME/.local/bin/corbanu" vault auth-helper neo-vm)" \
    bash -c 'printf "%s\n" "$NEOPW" | ssh "$@"' _ "${SSH_OPTS[@]}" "$G" "/usr/bin/expect -f ~/ag-admin/su.exp /Users/agent/ag-admin/$name"
}
rootsh() { local f; f=$(mktemp "$Q/logs/rootsh.XXXXXX"); { echo 'echo PF83BEGIN'; cat; } > "$f"; root "$f"; rm "$f"; }
STATE='echo "---agms"; ls ~/agms 2>&1 | head; ls ~/agms/restore-state 2>&1; echo "---"; ls /etc/codex 2>&1;
  echo "agmx accounts: $(dscl . list /Users | grep -c "^agmx")"; ls /opt; test -f ~/.pf83-auth/zai_api_key && echo keyfile-present;
  echo "AGMS pf lines: $(grep -c AGMS /etc/pf.anchors/corbanu.pf83)"; grep -n "skip on lo0" /etc/pf.anchors/corbanu.pf83;
  mount | grep -i -E "virtiofs|Shared"; ls ~/ag-admin 2>&1; ls -d ~/agms-canary /private/tmp/agms-canary-ipc /Users/Shared/agms-canary-ipc.pid /etc/sudoers.d/agms-codeblind 2>&1;
  echo "who: $(who | tr "\n" ";")"; ls /Users; date -u +%FT%TZ'

case "${1:-}" in
  up)
    [ ! -e "$LOCK" ] || grep -q '^holder: utm-par2$' "$LOCK" || die "UTM lock held: $(cat "$LOCK")"
    [ ! -e "$TLOCK" ] || die "Tart lock held (one VM at a time): $(cat "$TLOCK")"
    pgrep -f "tart run" > /dev/null && die "a Tart VM is running (one VM at a time)"
    "$U" list | grep -q 'stopped  macOS 2' || die "macOS 2 is not stopped; not starting anything"
    ls -d /Volumes/FASTDRIVE2/z23 > /dev/null || die "the VM's shared folder /Volumes/FASTDRIVE2/z23 is missing"
    mkdir -p "$C"
    printf 'holder: utm-par2\ntaken: %s\nvm: UTM "macOS 3" (D93 same-build parity run)\n' "$(date -u +%FT%TZ)" > "$LOCK"
    {
      date -u +%FT%TZ; "$U" list; memory_pressure -Q | tail -1
      "$U" start "$VM"; echo "start-rc=$?"
      for _ in $(seq 1 60); do nc -z -G 3 192.168.64.3 22 2> /dev/null && break; sleep 3; done
      echo "ssh open $(date -u +%FT%TZ)"; sleep 3
      ag 'sw_vers; uptime; csrutil status; spctl --status; echo "guest clock: $(date -u +%FT%TZ)"' 2>&1 || true
      echo "host clock: $(date -u +%FT%TZ)"
      ag "bash -s" < "$Q/h/env-manifest.sh"
      "$U" list
    } 2>&1 | tee "$C/vm-start.txt"
    ag "$STATE" > "$C/guest-state-start.txt" 2>&1 || true
    cat "$C/guest-state-start.txt"
    ;;
  tunnel)
    "${TM[@]}" kill-session -t tunnel 2> /dev/null || true
    q=""; for a in "${TUN_OPTS[@]}"; do q="$q '${a//\'/\'\\\'\'}'"; done
    "${TM[@]}" new-session -d -s tunnel \
      "while :; do echo \"\$(date -u +%FT%TZ) connect $G\" >> '$C/tunnel.log'; ssh $q $G >> '$C/tunnel.log' 2>&1; echo \"\$(date -u +%FT%TZ) ssh exited rc=\$?\" >> '$C/tunnel.log'; sleep 3; done"
    sleep 4
    ag 'lsof -nP -iTCP:18601 -iTCP:18602 -sTCP:LISTEN | awk "NR>1 {print \$1, \$3, \$9}"' | tee "$C/tunnel-listeners.txt"
    ;;
  stage)
    ag 'mkdir -p -m 700 ~/agms ~/agms/stage ~/agms/mediator; echo ok'
    COPYFILE_DISABLE=1 tar --no-xattrs -C "$Q/stage" -cf - package package-files.txt packet | agi 'cd ~/agms/stage && tar -xf - && echo staged-package-packet'
    items="agms-launch agms-probe agms-product agms-tui agms-mcp-notes agms_mediator.py tmux.conf mediation-ca.pem product-config.toml fixture.tar expected.json"
    COPYFILE_DISABLE=1 tar --no-xattrs -C "$Q/h" -czf - $items | agi 'mkdir -p ~/agms/stage/h && cd ~/agms/stage/h && tar -xzf - && echo staged-harness'
    agi 'umask 077; cat > ~/agms/mediator/leaf.pem' < "$Q/ca/leaf.pem"
    agi 'umask 077; cat > ~/agms/mediator/leaf.key' < "$Q/ca/leaf.key"
    agi 'bash -s' < "$Q/hu/setup-agent-utm.sh" > "$C/setup-agent.out" 2>&1 || die "setup-agent failed (see $C/setup-agent.out)"
    root "$Q/h/provision.sh" > "$C/provision.out" 2>&1 || die "provision failed (see $C/provision.out)"
    root "$Q/hu/erase-etc-codex.sh" > "$C/erase-etc-codex.out" 2>&1 || die "erase-etc-codex failed"
    root "$Q/h/capture-policy.sh" > "$C/policy-root-start.txt" 2>&1 || true
    tail -4 "$C/setup-agent.out"; tail -3 "$C/provision.out"; cat "$C/erase-etc-codex.out"
    ;;
  gate)
    O=$C/gate; mkdir -p "$O"
    R=gate-agmx0-$(date -u +%Y%m%dT%H%M%SZ); echo "$R" > "$O/current"
    ag 'for h in github.com raw.githubusercontent.com codeload.github.com example.com api.z.ai; do curl -sS -m 8 -o /dev/null https://$h 2>&1; echo "rc=$? $h"; done' > "$O/agent-egress.txt" 2>&1 || true
    ag "cd /tmp; sudo -n -u agmx0 -H /opt/agms/bin/agms-launch probe $R" > "$O/$R.summary" 2> "$O/$R.stderr" || true
    ag "cd /tmp; sudo -n -u agmx0 -H /opt/agms/bin/agms-launch smoke $R" > "$O/smoke-$R.jsonl" 2> "$O/smoke-$R.stderr" || true
    ag 'for u in https://api.z.ai/api/paas/v4/models; do
          printf "%s " "$u"; curl -sS -m 20 -o /dev/null -w "%{http_code}\n" --proxy http://127.0.0.1:18443 --cacert ~/agms/stage/h/mediation-ca.pem "$u"; done
        tail -3 ~/agms/mediator/mediator.jsonl | cut -c1-300' > "$O/relay-positive-controls.txt" 2>&1 || true
    cat "$O/$R.summary"; grep -o 'AGMS-SMOKE-OK' "$O/smoke-$R.jsonl" | head -1; cat "$O/relay-positive-controls.txt"
    ;;
  group)
    LANE="$2"; GROUP="$3"; LABEL="${4:-}"
    wipe() {
      ag "cd /tmp; sudo -n -u $LANE -H /bin/bash -c '
        pkill -u \$(id -u) -f \"^/opt/agms/(bin/tmux|pkg/)\" || true
        find /Users/$LANE -mindepth 1 -delete 2>/dev/null
        for d in \$(getconf DARWIN_USER_TEMP_DIR) \$(getconf DARWIN_USER_CACHE_DIR); do find \"\$d\" -mindepth 1 -user \$(id -u) -delete 2>/dev/null; done
        find /private/tmp /Users/Shared -maxdepth 3 -user \$(id -u) -delete 2>/dev/null
        echo wiped: \$(find /Users/$LANE -mindepth 1 | wc -l) entries left; ps -axo uid= | awk -v u=\$(id -u) \"\\\$1==u\" | wc -l | sed s/^/processes:/'"
    }
    RUN="agms-$GROUP${LABEL:+-$LABEL}-$LANE-$(date -u +%Y%m%dT%H%M%SZ)"
    OUT="$Q/runs/$RUN"; mkdir -p "$OUT"
    echo "== $RUN (evidence folder $C)" | tee -a "$Q/runs/seq.log"
    if ! python3 "$Q/h/gate_ok.py" "$C" > "$OUT/gate-check.txt" 2>&1; then
      echo "gate not passed ($C); group blocked" | tee "$OUT/BLOCKED" | tee -a "$Q/runs/seq.log"
      exit 0
    fi
    memory_pressure -Q | tail -1 > "$OUT/memory-before.txt"
    ag 'tail -1 ~/agms/guard.log; grep -c SECURITYAGENT-SEEN ~/agms/guard.log' > "$OUT/guard-before.txt" 2>&1 || true
    wipe > "$OUT/wipe-before.txt" 2>&1 || true
    ag "cd /tmp; sudo -n -u $LANE -H /opt/agms/bin/agms-launch probe $RUN" > "$OUT/preflight-summary.stdout" 2> "$OUT/preflight.stderr" || true
    if ! grep -q '"ok": true' "$OUT/preflight-summary.stdout"; then
      echo "preflight failed for $RUN; group blocked" | tee "$OUT/BLOCKED"
    else
      date -u +%FT%TZ > "$OUT/executor-start.txt"
      set +e
      ag "cd /tmp; sudo -n -u $LANE -H /opt/agms/bin/agms-launch executor $RUN $GROUP ${AGMS_EXECUTOR_MODEL:-glm-5.3-flash}" \
        > "$OUT/executor-events.jsonl" 2> "$OUT/executor.stderr"
      echo "exit=$?" > "$OUT/executor-exit.txt"
      set -e
      date -u +%FT%TZ > "$OUT/executor-end.txt"
    fi
    ag "cd /tmp; sudo -n -u $LANE -H /opt/agms/bin/agms-launch cleanup $RUN" > /dev/null 2>&1 || true
    ag "cd /tmp; sudo -n -u $LANE -H /opt/agms/bin/agms-launch collect $RUN" > "$OUT/collected.tar" 2> "$OUT/collect.stderr" || true
    ag 'cat ~/agms/mediator/mediator.jsonl' > "$OUT/mediator-at-end.jsonl" 2>&1 || true
    ag 'tail -1 ~/agms/guard.log; grep -c SECURITYAGENT-SEEN ~/agms/guard.log' > "$OUT/guard-after.txt" 2>&1 || true
    memory_pressure -Q | tail -1 > "$OUT/memory-after.txt"
    wipe > "$OUT/wipe-after.txt" 2>&1 || true
    echo "done $RUN $(cat "$OUT/executor-exit.txt" 2>/dev/null)" | tee -a "$Q/runs/seq.log"
    ;;
  status)
    ag 'tail -2 ~/agms/mediator/mediator.jsonl | cut -c1-300; tail -1 ~/agms/guard.log; grep -c SECURITYAGENT-SEEN ~/agms/guard.log'
    memory_pressure -Q | tail -1
    ;;
  down)
    # Logs and root policy; delete the synthetic test data (lane homes) and verify; restore.sh reverses provisioning
    # (accounts, sudoers, /opt/agms, pf, ~/agms with the mediator's synthetic leaf key, canaries, /etc/codex);
    # restore-fix.sh remounts the share; live check; stop the tunnel; clean shutdown (removes ~agent/ag-admin).
    {
      ag 'cat ~/agms/mediator/mediator.jsonl' > "$C/mediator-final.jsonl" 2>/dev/null || true
      ag 'cat ~/agms/guard-baseline.txt' > "$C/guard-baseline.txt" 2>/dev/null || true
      ag 'tail -1 ~/agms/guard.log; echo "SECURITYAGENT-SEEN lines: $(grep -c SECURITYAGENT-SEEN ~/agms/guard.log)"; echo "checks: $(wc -l < ~/agms/guard.log)"' > "$C/guard-final.txt" 2>/dev/null || true
      root "$Q/h/capture-policy.sh" > "$C/policy-root-end.txt" 2>&1 || true
      printf '%s\n' 'for u in agmx0 agmx1 agmx2 agmx3; do pkill -u $u || true; done; sleep 1; for u in agmx0 agmx1 agmx2 agmx3; do echo "$u processes: $(ps -axo user= | grep -c "^$u$")"; done' | rootsh > "$C/kill-lane-procs-end.out" 2>&1 || true
      printf '%s\n' 'for u in agmx0 agmx1 agmx2 agmx3; do [ ! -d /Users/$u ] || find /Users/$u -mindepth 1 -delete; done' \
        'for u in agmx0 agmx1 agmx2 agmx3; do echo "$u home entries left: $(find /Users/$u -mindepth 1 2>/dev/null | wc -l | tr -d " ")"; done' \
        'echo "synthetic credential files left in lane homes: $(find /Users/agmx0 /Users/agmx1 /Users/agmx2 /Users/agmx3 \( -name credentials -o -name .netrc -o -name .env -o -name config -path "*/.ssh/*" \) -print 2>/dev/null | wc -l | tr -d " ")"' \
        | rootsh > "$C/erase-test-data.out" 2>&1 || echo "TEST DATA DELETION FAILED" >> "$C/erase-test-data.out"
      cat "$C/erase-test-data.out"
      root "$Q/hu/restore.sh" > "$C/restore-end.out" 2>&1 || true
      root "$Q/hu/restore-fix.sh" > "$C/restore-fix-end.out" 2>&1 || true
      grep -E '^\[restore' "$C/restore-end.out" "$C/restore-fix-end.out" || true
      ag "echo \"live check after restore (before shutdown)\"; $STATE; test ! -e ~/agms/mediator/leaf.key && echo 'mediator leaf key: absent'" > "$C/vm-state-end.txt" 2>&1 || true
      cat "$C/vm-state-end.txt"
      "${TM[@]}" kill-session -t tunnel 2> /dev/null && echo "tunnel stopped" || echo "tunnel not running"
      echo "$(date -u +%FT%TZ) guest-shutdown-clean.sh"
      root "$Q/hu/guest-shutdown-clean.sh" 2>&1 || true
      for _ in $(seq 1 40); do "$U" list | grep -q 'stopped  macOS 3' && break; sleep 3; done
      "$U" list; date -u +%FT%TZ
    } 2>&1 | tee "$C/vm-stop.txt"
    ;;
  verify-boot)
    # Start once more to confirm ~agent/ag-admin and every campaign path are gone; then stop; free the lock.
    {
      echo "$(date -u +%FT%TZ) verification boot: utmctl start"; "$U" start "$VM"
      for _ in $(seq 1 60); do nc -z -G 3 192.168.64.3 22 2> /dev/null && break; sleep 3; done
      echo "$(date -u +%FT%TZ) ssh reachable: $(nc -z -G 3 192.168.64.3 22 2>/dev/null && echo yes || echo no)"; sleep 5
      ag "$STATE" 2>&1 || true
      "$U" stop "$VM" --request 2>&1 || true
      for _ in $(seq 1 40); do "$U" list | grep -q 'stopped  macOS 3' && break; sleep 3; done
      "$U" list | grep -q 'stopped  macOS 3' || { echo "$(date -u +%FT%TZ) shutdown request not answered in 2 min; utmctl stop"; "$U" stop "$VM"; sleep 5; }
      "$U" list; date -u +%FT%TZ
    } 2>&1 | tee "$C/vm-state-end2.txt"
    "$U" list | grep -q 'stopped  macOS 3' && rm "$LOCK" && echo "lock released"
    ;;
  *) echo "usage: utm-run.sh up|tunnel|stage|gate|group LANE GROUP [LABEL]|status|down|verify-boot" >&2; exit 64 ;;
esac
