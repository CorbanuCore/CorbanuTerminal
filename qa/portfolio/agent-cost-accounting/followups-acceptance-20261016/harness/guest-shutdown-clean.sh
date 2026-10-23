#!/bin/bash
# Run as root (via admin.sh) at the very end: remove the admin helper folder (this script and su.exp included), then shut down.
echo PF83BEGIN
rm -r /Users/agent/ag-admin && echo "ag-admin removed"
sync; sync
/sbin/shutdown -h +0 && echo shutdown-requested
