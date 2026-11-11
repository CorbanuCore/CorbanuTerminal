echo PF83BEGIN
set -e
cmp -s /etc/codex/config.toml /Users/agent/agms/restore-state/etc-codex/config.toml && rm /etc/codex/config.toml && rmdir /etc/codex
ls -la /etc/codex 2>&1 | head -1
