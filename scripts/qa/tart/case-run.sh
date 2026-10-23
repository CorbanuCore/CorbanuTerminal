#!/bin/bash
# Run frozen code-blind case groups in fresh Tart clones (one clone per group), using the guest-side
# files of a UTM milestone harness unchanged. Nothing is shared from the host: the package, packet
# and harness go in over SSH, and the clone is deleted after the group.
#
#   TART_QA_WORK=... HARNESS=DIR PACKAGE=DIR PACKET=DIR scripts/qa/tart/case-run.sh GROUP...
#
#   HARNESS  guest-side harness: provision.sh setup-agent.sh agms-launch agms-probe agms-product agms-tui
#            agms-mcp-notes agms_mediator.py tmux.conf product-config.toml fixture.tar
#   PACKAGE  the exact candidate package (bin/corbanu ...); PACKET the frozen executor packet
#            (design-cases.md, amendments.md, prompts/GROUP.md, ...).
#   TMUX_BIN (default: Corbanu's pinned tmux 3.7c) and its libevent_core-2.1.7.dylib.
#
# Per group: clone -> boot -> seal-guest.sh -> stage -> provision.sh (root) -> setup-agent.sh -> isolation
# gate (probes + smoke on lane agmx0) -> preflight + fresh executor on lane agmx1 -> collect -> delete the
# transport key file -> delete the clone. The model key is read from the vault at use time and written only to the
# guest coordinator's 0600 key file, which the mediator reads; executors never see it.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=SCRIPTDIR/lib.sh
. "$here/lib.sh"
: "${HARNESS:?}" "${PACKAGE:?}" "${PACKET:?}"
[ $# -ge 1 ] || die "usage: case-run.sh GROUP..."
# Defaults are this Mac's pinned tmux and Homebrew libevent; set both elsewhere.
TMUX_BIN="${TMUX_BIN:-/Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/tools/tmux-source/3.7c/bin/tmux}"
LIBEVENT="${LIBEVENT:-/opt/homebrew/opt/libevent/lib/libevent_core-2.1.7.dylib}"
LABEL="${TART_QA_ZAI_LABEL:-provider/zai_api_key}"
CA="$TART_QA_WORK/ca"
OUT_ROOT="$TART_QA_WORK/case-runs"
mkdir -p "$OUT_ROOT"

for f in provision.sh setup-agent.sh agms-launch agms-probe agms-product agms-tui agms-mcp-notes agms_mediator.py \
  tmux.conf product-config.toml fixture.tar; do
  [ -f "$HARNESS/$f" ] || die "missing $HARNESS/$f"
done
codesign --verify --strict "$PACKAGE/bin/corbanu" || die "package binary not validly signed"

# Campaign-local synthetic CA for the mediator's api.z.ai leaf (created once, never leaves this host
# except the leaf, which goes to the guest coordinator account only).
if [ ! -f "$CA/leaf.pem" ]; then
  mkdir -p "$CA" && chmod 700 "$CA"
  (
    cd "$CA"
    openssl req -x509 -newkey rsa:2048 -nodes -keyout ca.key -out ca.pem -days 30 -subj "/CN=Tart QA mediation CA (synthetic)" 2> /dev/null
    openssl req -newkey rsa:2048 -nodes -keyout leaf.key -out leaf.csr -subj "/CN=api.z.ai" 2> /dev/null
    printf '%s\n' 'subjectAltName=DNS:api.z.ai' 'extendedKeyUsage=serverAuth' > leaf.ext
    openssl x509 -req -in leaf.csr -CA ca.pem -CAkey ca.key -CAcreateserial -out leaf.pem -days 30 -extfile leaf.ext 2> /dev/null
    chmod 600 ca.key leaf.key
  )
fi

# Stage once: package + manifest, harness, packet, expected digests.
STAGE="$TART_QA_WORK/stage"
rm -rf "$STAGE"; mkdir -p "$STAGE/h"
cp -Rp "$PACKAGE" "$STAGE/package"
(cd "$STAGE/package" && find . -type f | sort | while read -r f; do
  printf '%s %s %s\n' "$(stat -f %Lp "$f")" "$(shasum -a 256 "$f" | cut -d' ' -f1)" "$f"
done) > "$STAGE/package-files.txt"
for f in agms-launch agms-probe agms-product agms-tui agms-mcp-notes agms_mediator.py tmux.conf product-config.toml fixture.tar; do
  cp -p "$HARNESS/$f" "$STAGE/h/"
done
cp "$CA/ca.pem" "$STAGE/h/mediation-ca.pem"
cp -Rp "$PACKET" "$STAGE/packet"
BIN_SHA="$(shasum -a 256 "$PACKAGE/bin/corbanu" | cut -d' ' -f1)"
PACKET_SHA="$(shasum -a 256 "$PACKET/design-cases.md" | cut -d' ' -f1)"
printf '{"binary_sha256": "%s", "packet_sha256": "%s"}' "$BIN_SHA" "$PACKET_SHA" > "$STAGE/h/expected.json"
SEAL="$TART_QA_WORK/seal-assets"
rm -rf "$SEAL"; mkdir -p "$SEAL"
cp "$here/seal-guest.sh" "$here/dns-refuse.py" "$TMUX_BIN" "$LIBEVENT" "$SEAL/"
cp "$TART_QA_KEY.pub" "$SEAL/coordinator.pub"

# shellcheck disable=SC2029  # remote commands are built on purpose
gssh_agent() { local ip="$1"; shift; ssh "${SSH_OPTS[@]}" "agent@$ip" "$@"; }
# No stdin (ssh -n): `corbanu exec` would otherwise wait for more prompt input on it.
# shellcheck disable=SC2029
gagent() { local ip="$1"; shift; ssh -n "${SSH_OPTS[@]}" "agent@$ip" "$@"; }
zai_key() {
  /usr/bin/env -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME -u CORBANU_TEST_NO_NATIVE_KEYRING \
    HOME="$HOME" "$HOME/.local/bin/corbanu" vault auth-helper "$LABEL"
}
stamp() { printf '%s %s\n' "$(date -u +%FT%TZ)" "$1" >> "$OUT/timeline.txt"; }

CURRENT="" IP=""
delete_key_and_clone() {
  [ -n "$CURRENT" ] || return 0
  if [ -n "$IP" ]; then
    # Deleting the file (no secure overwrite on APFS); its blocks go with the clone, deleted next.
    gssh "$IP" 'sudo rm -f /Users/agent/.pf83-auth/zai_api_key; sudo test ! -e /Users/agent/.pf83-auth/zai_api_key && echo "transport key file deleted"' \
      >> "$OUT/erase.txt" 2>&1 || echo "key file deletion over SSH failed" >> "$OUT/erase.txt"
  fi
  stop_vm "$CURRENT" 10
  "$TART" delete "$CURRENT"
  stamp deleted
  CURRENT="" IP=""
}
trap delete_key_and_clone EXIT

run_group() {
  local group="$1" run lane=agmx1 gate
  OUT="$OUT_ROOT/$group-$(date -u +%Y%m%dT%H%M%SZ)"
  mkdir -p "$OUT"
  CURRENT="qa-case-$(basename "$OUT")"
  stamp "clone $CURRENT"
  "$TART" clone "$TART_QA_BASE" "$CURRENT"
  start_vm "$CURRENT"
  IP="$(wait_ip "$CURRENT" 180)"
  wait_ssh "$IP" 180 || die "no SSH on $IP"
  stamp "ssh ready $IP"

  tar -C "$SEAL" -czf - . | gssh "$IP" 'rm -rf /tmp/qa-seal && mkdir -m 700 /tmp/qa-seal && tar -xzf - -C /tmp/qa-seal'
  gssh "$IP" 'sudo bash /tmp/qa-seal/seal-guest.sh /tmp/qa-seal && rm -rf /tmp/qa-seal' > "$OUT/seal.out" 2>&1
  stamp sealed
  # shellcheck disable=SC2016
  printf '%s\n' "$(zai_key)" | gssh "$IP" 'sudo -u agent /bin/bash -c "umask 077; cat > /Users/agent/.pf83-auth/zai_api_key"'
  COPYFILE_DISABLE=1 tar --no-xattrs -C "$STAGE" -czf - . | gssh_agent "$IP" 'mkdir -p ~/agms/stage && tar -xzf - -C ~/agms/stage'
  gssh_agent "$IP" 'umask 077; cat > ~/agms/mediator/leaf.pem' < "$CA/leaf.pem"
  gssh_agent "$IP" 'umask 077; cat > ~/agms/mediator/leaf.key' < "$CA/leaf.key"
  stamp staged
  gssh "$IP" 'sudo bash -s' < "$HARNESS/provision.sh" > "$OUT/provision.out" 2>&1 || die "provision failed (see $OUT/provision.out)"
  gssh_agent "$IP" 'bash -s' < "$HARNESS/setup-agent.sh" > "$OUT/setup-agent.out" 2>&1 || die "setup-agent failed"
  stamp provisioned

  gate="gate-$(date -u +%Y%m%dT%H%M%SZ)"
  gagent "$IP" "cd /tmp; sudo -n -u agmx0 -H /opt/agms/bin/agms-launch probe $gate" > "$OUT/gate.summary" 2> "$OUT/gate.stderr" || true
  gagent "$IP" "cd /tmp; sudo -n -u agmx0 -H /opt/agms/bin/agms-launch smoke $gate" > "$OUT/gate-smoke.jsonl" 2> "$OUT/gate-smoke.stderr" || true
  stamp "gate $(grep -o '"ok": [a-z]*' "$OUT/gate.summary" | tail -1); smoke $(grep -c AGMS-SMOKE-OK "$OUT/gate-smoke.jsonl")"
  if ! grep -q '"ok": true' "$OUT/gate.summary" || ! grep -q AGMS-SMOKE-OK "$OUT/gate-smoke.jsonl"; then
    echo "isolation gate or smoke failed; group blocked" > "$OUT/BLOCKED"
    delete_key_and_clone
    return 0
  fi

  run="agms-$group-$lane-$(date -u +%Y%m%dT%H%M%SZ)"
  gagent "$IP" "cd /tmp; sudo -n -u $lane -H /opt/agms/bin/agms-launch probe $run" > "$OUT/preflight-summary.stdout" 2> "$OUT/preflight.stderr" || true
  if grep -q '"ok": true' "$OUT/preflight-summary.stdout"; then
    stamp "executor start $run"
    gagent "$IP" "cd /tmp; sudo -n -u $lane -H /opt/agms/bin/agms-launch executor $run $group ${AGMS_EXECUTOR_MODEL:-glm-5.3-flash}" \
      > "$OUT/executor-events.jsonl" 2> "$OUT/executor.stderr" && echo "exit=0" > "$OUT/executor-exit.txt" || echo "exit=$?" > "$OUT/executor-exit.txt"
    stamp "executor end $(cat "$OUT/executor-exit.txt")"
  else
    echo "preflight failed for $run; group blocked" > "$OUT/BLOCKED"
  fi
  gagent "$IP" "cd /tmp; sudo -n -u $lane -H /opt/agms/bin/agms-launch cleanup $run" > /dev/null 2>&1 || true
  gagent "$IP" "cd /tmp; sudo -n -u $lane -H /opt/agms/bin/agms-launch collect $run" > "$OUT/collected.tar" 2> "$OUT/collect.stderr" || true
  gagent "$IP" 'cat ~/agms/mediator/mediator.jsonl' > "$OUT/mediator.jsonl" 2>&1 || true
  gagent "$IP" 'tail -1 ~/agms/guard.log; grep -c SECURITYAGENT-SEEN ~/agms/guard.log' > "$OUT/guard.txt" 2>&1 || true
  stamp collected
  delete_key_and_clone
}

for group in "$@"; do run_group "$group"; done
