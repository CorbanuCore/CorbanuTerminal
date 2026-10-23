echo PF83BEGIN
# Suspend 'set skip on lo0' for the campaign so the per-uid loopback rules apply (restored by restore.sh).
set -e
grep -q '^set skip on lo0$' /etc/pf.anchors/corbanu.pf83
sed -i '' 's/^set skip on lo0$/# set skip on lo0   (suspended by AGMS 2026-10-10; restore.sh puts it back)/' /etc/pf.anchors/corbanu.pf83
pfctl -nf /etc/pf.conf
pfctl -f /etc/pf.conf 2>&1 | grep -v -e ALTQ -e "flushing of rules" -e "present in the main" -e "pf.conf for further" -e '^$' || true
echo "[lo0] $(pfctl -sI -v 2>/dev/null | grep lo0 || echo 'lo0 not skipped')"
pfctl -a corbanu.pf83 -sr | head -4
