echo PF83BEGIN
echo "== anchors"; pfctl -sA 2>/dev/null
echo "== main rules"; pfctl -sr 2>/dev/null | head -20
echo "== corbanu.pf83"; pfctl -a corbanu.pf83 -sr 2>/dev/null
echo "== anchor file"; cat /etc/pf.anchors/corbanu.pf83
echo "== skip"; pfctl -sI -v 2>/dev/null | head -5; grep -n skip /etc/pf.conf
echo "== hosts"; grep -v '^#' /etc/hosts
echo "== users"; dscl . -list /Users UniqueID | awk '$2>=500'
echo "== opt"; ls -la /opt 2>&1; ls /etc/sudoers.d
echo "== mount"; mount | grep -i -E "virtiofs|Shared" || echo none
echo "== homes"; stat -f '%N %Lp %Su' /Users/* 
echo "== tools"; ls -la /Users/agent/bin; which python3 expect; python3 --version
echo "== agent qual"; ls -la /Users/agent/qual | head
