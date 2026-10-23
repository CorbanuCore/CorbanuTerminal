#!/bin/bash
# acc-tart1 host-side driver: the PF-60-S10 harness on one fresh Tart clone of corbanu-qa-base (D34), replacing S10's
# UTM scripts (vm-up.sh, vmssh.sh, admin.sh, stage-and-provision.sh, gate.sh, run_group.sh, restore*.sh).
#   tart-run.sh up | seal | stage | gate | group LANE GROUP [LABEL] | moonshot-enable | status | down
# One Tart VM at a time (scripts/qa/tart/lib.sh guard_slot); UTM is never touched. Real keys are read from the vault at
# use time and piped straight into agent-only files on the guest; they are deleted before the clone is deleted.
set -euo pipefail
W=/Volumes/CorbanuDrive/Corbanu/.codex-work/acc-tart1-20261020
WT=/Volumes/CorbanuDrive/Corbanu/worktrees/acc-tart1
WORKERS=/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002
export TART_QA_WORK=/Volumes/CorbanuDrive/Corbanu/.codex-work/tart-eval   # keys + known_hosts of the base
export TART_HOME=/Volumes/CorbanuDrive/Corbanu/.codex-work/tart-home
export TART=/Volumes/CorbanuDrive/Corbanu/.codex-work/tart-bin/tart.app/Contents/MacOS/tart
# shellcheck source=/dev/null
. "$WT/scripts/qa/tart/lib.sh"
VM=acc-tart1-run
LOCK=$WORKERS/vm-tart.lock
C=$W/common; mkdir -p "$C" "$W/runs"
ip_of() { cat "$W/common/vm-ip.txt"; }
ag() { ssh -n "${SSH_OPTS[@]}" "agent@$(ip_of)" "$@"; }         # coordinator account, no stdin
agi() { ssh "${SSH_OPTS[@]}" "agent@$(ip_of)" "$@"; }           # coordinator account, stdin
root() { gssh "$(ip_of)" "sudo bash -s"; }                       # image admin -> root, script on stdin
vault() { /usr/bin/env -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME -u CORBANU_TEST_NO_NATIVE_KEYRING \
  HOME="$HOME" "$HOME/.local/bin/corbanu" vault auth-helper "$1"; }
place_key() { # LABEL FILE: vault value -> agent-only file on the guest; never printed or written on the host
  K="$(vault "$1")" bash -c 'printf "%s" "$K"' | agi "umask 077; mkdir -p ~/.pf83-auth; cat > ~/.pf83-auth/$2; test -s ~/.pf83-auth/$2 && echo $2 placed"
}

case "${1:-}" in
  up)
    [ ! -e "$LOCK" ] || die "Tart lock held: $(cat "$LOCK")"
    export TART_HOME TMPDIR
    printf 'holder: acc-tart1\ntaken: %s\nvm: Tart %s (clone of %s)\n' "$(date -u +%FT%TZ)" "$VM" "$TART_QA_BASE" > "$LOCK"
    {
      date -u +%FT%TZ; "$TART" list; "$UTMCTL" list 2>&1 || true
      "$TART" clone "$TART_QA_BASE" "$VM"
      # lib.sh's start_vm backgrounds `tart run` with nohup; a tool shell that exits takes it down, so the VM
      # runs in its own session on this run's tmux server instead (no host shares, audio or clipboard).
      guard_slot
      /opt/homebrew/bin/tmux -S "$W/tmux.sock" new-session -d -s vm \
        "exec '$TART' run --no-graphics --no-audio --no-clipboard $VM > '$LOGS/$VM.run.log' 2>&1"
      sleep 2
      pgrep -f "tart run --no-graphics --no-audio --no-clipboard $VM" | head -1 > "$LOGS/$VM.pid"
      echo "tart run pid $(cat "$LOGS/$VM.pid")"
      ip="$(wait_ip "$VM" 180)"; echo "$ip" > "$C/vm-ip.txt"
      wait_ssh "$ip" 180 || die "no SSH on $ip"
      echo "ssh ready $(date -u +%FT%TZ) $ip"
      gssh "$ip" 'sw_vers; uptime; csrutil status; spctl --status'
      "$TART" list
    } 2>&1 | tee "$C/vm-start.txt"
    ;;
  seal)
    A=$W/seal-assets; [ ! -e "$A" ] || rm -r "$A" < /dev/null; mkdir -p "$A"
    cp "$WT/scripts/qa/tart/seal-guest.sh" "$WT/scripts/qa/tart/dns-refuse.py" "$A/"
    cp /Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/tools/tmux-source/3.7c/bin/tmux "$A/"
    cp /opt/homebrew/opt/libevent/lib/libevent_core-2.1.7.dylib "$A/"
    cp "$TART_QA_KEY.pub" "$A/coordinator.pub"
    tar -C "$A" -czf - . | gssh "$(ip_of)" '{ [ ! -e /tmp/qa-seal ] || rm -r /tmp/qa-seal; } && mkdir -m 700 /tmp/qa-seal && tar -xzf - -C /tmp/qa-seal'
    gssh "$(ip_of)" 'sudo bash /tmp/qa-seal/seal-guest.sh /tmp/qa-seal && rm -r /tmp/qa-seal' 2>&1 | tee "$C/seal.out"
    ;;
  stage)
    . "$W/caps.env"   # MOONSHOT_CAP REALTIME_CAP (owner decisions; see the record)
    ag 'mkdir -p -m 700 ~/agms ~/agms/stage ~/agms/mediator; echo ok'
    COPYFILE_DISABLE=1 tar --no-xattrs -C "$W/stage" -cf - package package-files.txt packet | agi 'cd ~/agms/stage && tar -xf - && echo staged-package-packet'
    items="agms-launch agms-probe agms-product agms-tui tmux.conf mediation-ca.pem product-config.toml fixture.tar expected.json acc_mediator.py acc_standin.py chatgpt_standin.py realtime_driver.py"
    COPYFILE_DISABLE=1 tar --no-xattrs -C "$W/h" -czf - $items | agi 'mkdir -p ~/agms/stage/h && cd ~/agms/stage/h && tar -xzf - && echo staged-harness'
    agi 'umask 077; cat > ~/agms/mediator/leaf.pem' < "$W/ca/leaf.pem"
    agi 'umask 077; cat > ~/agms/mediator/leaf.key' < "$W/ca/leaf.key"
    printf 'MOONSHOT_CAP=%s\nREALTIME_CAP=%s\n' "$MOONSHOT_CAP" "$REALTIME_CAP" | agi 'umask 077; cat > ~/agms/caps'
    place_key provider/zai_api_key zai_api_key
    place_key openai-api-key openai_api_key
    if [ "$MOONSHOT_CAP" -gt 0 ]; then place_key kimi_api3 kimi_api_key; else echo "kimi key not placed (Moonshot cap 0)"; fi
    agi 'bash -s' < "$W/h/setup-agent.sh" > "$C/setup-agent.out" 2>&1 || die "setup-agent failed (see $C/setup-agent.out)"
    root < "$W/h/provision.sh" > "$C/provision.out" 2>&1 || die "provision failed (see $C/provision.out)"
    root < "$W/h/capture-policy.sh" > "$C/policy-root-start.txt" 2>&1 || true
    tail -4 "$C/setup-agent.out"; tail -3 "$C/provision.out"
    ;;
  moonshot-enable)
    # Owner approved Moonshot spend after staging: place the key, raise the cap, restart the relay (counters reset).
    . "$W/caps.env"
    [ "$MOONSHOT_CAP" -gt 0 ] || die "caps.env still has MOONSHOT_CAP=0"
    place_key kimi_api3 kimi_api_key
    printf 'MOONSHOT_CAP=%s\nREALTIME_CAP=%s\n' "$MOONSHOT_CAP" "$REALTIME_CAP" | agi 'umask 077; cat > ~/agms/caps'
    ag 'cp ~/agms/mediator/mediator.jsonl ~/agms/mediator/mediator-before-moonshot-enable.jsonl'
    agi 'bash -s' < "$W/h/setup-agent.sh" > "$C/setup-agent-moonshot-enable.out" 2>&1
    tail -4 "$C/setup-agent-moonshot-enable.out"
    ;;
  gate)
    O=$C/gate; mkdir -p "$O"
    R=gate-agmx0-$(date -u +%Y%m%dT%H%M%SZ)
    ag 'for h in github.com raw.githubusercontent.com codeload.github.com example.com; do curl -sS -m 8 -o /dev/null https://$h 2>&1; echo "rc=$? $h"; done' > "$O/agent-egress.txt" 2>&1 || true
    ag "cd /tmp; sudo -n -u agmx0 -H /opt/agms/bin/agms-launch probe $R" > "$O/$R.summary" 2> "$O/$R.stderr" || true
    ag "cd /tmp; sudo -n -u agmx0 -H /opt/agms/bin/agms-launch smoke $R" > "$O/smoke-$R.jsonl" 2> "$O/smoke-$R.stderr" || true
    # Relay positive controls as the coordinator account (free GETs; no inference).
    ag 'for u in https://api.openai.com/v1/models https://api.moonshot.ai/v1/models; do
          printf "%s " "$u"; curl -sS -m 20 -o /dev/null -w "%{http_code}\n" --proxy http://127.0.0.1:18443 --cacert ~/agms/stage/h/mediation-ca.pem "$u"; done
        printf "direct 127.0.0.1:443 GET /v1/models "; curl -sS -m 20 -o /dev/null -w "%{http_code}\n" --cacert ~/agms/stage/h/mediation-ca.pem https://api.openai.com/v1/models
        tail -4 ~/agms/mediator/mediator.jsonl | cut -c1-300' > "$O/relay-positive-controls.txt" 2>&1 || true
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
    RUN="acc-$GROUP${LABEL:+-$LABEL}-$LANE-$(date -u +%Y%m%dT%H%M%SZ)"
    OUT="$W/runs/$RUN"; mkdir -p "$OUT"
    echo "== $RUN" | tee -a "$W/runs/seq.log"
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
    wipe > "$OUT/wipe-after.txt" 2>&1 || true
    echo "done $RUN $(cat "$OUT/executor-exit.txt" 2>/dev/null)" | tee -a "$W/runs/seq.log"
    ;;
  status)
    ag 'tail -2 ~/agms/mediator/mediator.jsonl | cut -c1-300; tail -1 ~/agms/guard.log; grep -c SECURITYAGENT-SEEN ~/agms/guard.log'
    ;;
  down)
    # Closing: logs and root policy, then delete every real key file and verify, stop, delete the clone, free the lock.
    {
      ag 'cat ~/agms/mediator/mediator.jsonl' > "$C/mediator-final.jsonl" 2>/dev/null || true
      ag 'cat ~/agms/mediator/standin.jsonl' > "$C/standin-final.jsonl" 2>/dev/null || true
      ag 'cat ~/agms/mediator/chatgpt-standin.jsonl' > "$C/chatgpt-standin-final.jsonl" 2>/dev/null || true
      ag 'cat ~/agms/guard-baseline.txt' > "$C/guard-baseline.txt" 2>/dev/null || true
      ag 'tail -1 ~/agms/guard.log; echo "SECURITYAGENT-SEEN lines: $(grep -c SECURITYAGENT-SEEN ~/agms/guard.log)"; echo "checks: $(wc -l < ~/agms/guard.log)"' > "$C/guard-final.txt" 2>/dev/null || true
      root < "$W/h/capture-policy.sh" > "$C/policy-root-end.txt" 2>&1 || true
      printf '%s\n' 'for u in agmx0 agmx1 agmx2 agmx3; do pkill -u $u || true; done; sleep 1; for u in agmx0 agmx1 agmx2 agmx3; do echo "$u processes: $(ps -axo user= | grep -c "^$u$")"; done' | root > "$C/kill-lane-procs-end.out" 2>&1 || true
      printf '%s\n' 'for k in zai_api_key openai_api_key kimi_api_key; do [ ! -e /Users/agent/.pf83-auth/$k ] || rm /Users/agent/.pf83-auth/$k; done' \
        'for k in zai_api_key openai_api_key kimi_api_key; do test ! -e /Users/agent/.pf83-auth/$k && echo "$k: absent"; done' \
        'ls -la /Users/agent/.pf83-auth' | root > "$C/erase-keys.out" 2>&1 || echo "KEY DELETION FAILED" >> "$C/erase-keys.out"
      cat "$C/erase-keys.out"
      date -u +%FT%TZ; stop_vm "$VM" 10; "$TART" delete "$VM"; echo "deleted $VM"; "$TART" list
      date -u +%FT%TZ
    } 2>&1 | tee "$C/vm-state-end.txt"
    rm "$LOCK" && echo "lock released"
    ;;
  *) echo "usage: tart-run.sh up|seal|stage|gate|group LANE GROUP [LABEL]|moonshot-enable|status|down" >&2; exit 64 ;;
esac
