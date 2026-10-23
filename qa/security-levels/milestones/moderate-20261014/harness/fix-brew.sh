echo PF83BEGIN
set -e
ST=/Users/agent/agms/restore-state
# modes.txt recorded /opt/homebrew as 755 before provisioning; re-record bin/sbin originals.
stat -f '%N %Lp %Su:%Sg' /opt/homebrew/bin /opt/homebrew/sbin >> "$ST/modes.txt"
chmod 755 /opt/homebrew
chmod 750 /opt/homebrew/bin /opt/homebrew/sbin
cat "$ST/modes.txt"
stat -f '%N %Lp' /opt/homebrew /opt/homebrew/bin /opt/homebrew/sbin
