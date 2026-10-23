echo PF83BEGIN
set -e
ST=/Users/agent/agms/restore-state
mkdir -p /etc/codex; cp -p "$ST/etc-codex/config.toml" /etc/codex/config.toml
ls -la /etc/codex/config.toml; shasum -a 256 /etc/codex/config.toml
