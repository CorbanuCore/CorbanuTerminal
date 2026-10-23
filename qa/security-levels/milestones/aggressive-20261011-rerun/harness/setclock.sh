echo PF83BEGIN
echo "[clock] guest before: $(date -u +%FT%TZ)  host: 2026-10-10T22:35:18Z"
date -u 101022352026.18 >/dev/null
echo "[clock] guest after: $(date -u +%FT%TZ)"
