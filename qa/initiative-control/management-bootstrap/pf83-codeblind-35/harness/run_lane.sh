#!/bin/bash
# Host-side coordinator for one isolation lane. Usage: run_lane.sh LANE CASE [CASE...]
# Each case: wipe lane state -> preflight probes -> fresh executor -> cleanup -> collect -> wipe.
set -uo pipefail
LANE="$1"; shift
H=/Volumes/CorbanuDrive/Corbanu/.codex-work/functional-pf83.20260915/increment-35-codeblind
SSH=(/tmp/pf83ssh.sh -n agent@192.168.64.3)
wipe() {
  "${SSH[@]}" "cd /tmp; sudo -n -u $LANE -H /bin/bash -c '
    pkill -u \$(id -u) -f \"^/opt/pf83/(bin/tmux|pkg/corbanu)\" || true
    find /Users/$LANE -mindepth 1 -delete 2>/dev/null
    for d in \$(getconf DARWIN_USER_TEMP_DIR) \$(getconf DARWIN_USER_CACHE_DIR); do find \"\$d\" -mindepth 1 -user \$(id -u) -delete 2>/dev/null; done
    find /private/tmp /Users/Shared -maxdepth 3 -user \$(id -u) -delete 2>/dev/null
    echo wiped: \$(find /Users/$LANE -mindepth 1 | wc -l) entries left'"
}
for CASE in "$@"; do
  RUN="cb35-$CASE-$LANE-$(date -u +%Y%m%dT%H%M%SZ)"
  OUT="$H/runs/$RUN"; mkdir -p "$OUT"
  echo "== $RUN"
  wipe > "$OUT/wipe-before.txt" 2>&1
  "${SSH[@]}" "cd /tmp; sudo -n -u $LANE -H /opt/pf83/bin/pf83-launch probe $RUN" > "$OUT/preflight-summary.stdout" 2> "$OUT/preflight.stderr"
  if ! grep -q '"ok": true' "$OUT/preflight-summary.stdout"; then
    echo "preflight failed for $RUN; case blocked" | tee "$OUT/BLOCKED"
  else
    date -u +%FT%TZ > "$OUT/executor-start.txt"
    "${SSH[@]}" "cd /tmp; sudo -n -u $LANE -H /opt/pf83/bin/pf83-launch executor $RUN $CASE" \
      > "$OUT/executor-events.jsonl" 2> "$OUT/executor.stderr"
    echo "exit=$?" > "$OUT/executor-exit.txt"
    date -u +%FT%TZ > "$OUT/executor-end.txt"
  fi
  "${SSH[@]}" "cd /tmp; sudo -n -u $LANE -H /opt/pf83/bin/pf83-launch cleanup $RUN" > /dev/null 2>&1
  "${SSH[@]}" "cd /tmp; sudo -n -u $LANE -H /opt/pf83/bin/pf83-launch collect $RUN" > "$OUT/collected.tar" 2> "$OUT/collect.stderr"
  "${SSH[@]}" "cat ~/pf83-cb/mediator/mediator.jsonl" > "$OUT/mediator-at-end.jsonl"
  wipe > "$OUT/wipe-after.txt" 2>&1
  echo "done $RUN $(cat "$OUT/executor-exit.txt" 2>/dev/null)"
done
