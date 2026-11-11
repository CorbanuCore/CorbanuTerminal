echo PF83BEGIN
grep -v '^#' /etc/pf.conf | grep -v '^$'
echo ==; head -20 /etc/pf.anchors/corbanu.pf83
echo ==; ls -la /etc/pf.anchors/ | head
