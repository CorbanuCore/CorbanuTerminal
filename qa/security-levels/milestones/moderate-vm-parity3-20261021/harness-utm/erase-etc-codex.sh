echo PF83BEGIN
set -e
ST=/Users/agent/agms/restore-state
# Leftover system config layer from an earlier harness (2026-09-17); it changes product behaviour. Moved aside, restored by restore.sh.
if [ -f /etc/codex/config.toml ]; then mkdir -p "$ST/etc-codex"; mv /etc/codex/config.toml "$ST/etc-codex/config.toml"; rmdir /etc/codex; fi
ls -la /etc/codex 2>&1 | head -1
# Other system-wide Codex/Corbanu layers that could leak in:
ls -la "/Library/Managed Preferences" 2>&1 | grep -i -E "codex|corbanu" || echo "no managed prefs for codex/corbanu"
ls /etc | grep -i -E "codex|corbanu" || echo "no other /etc entries"
