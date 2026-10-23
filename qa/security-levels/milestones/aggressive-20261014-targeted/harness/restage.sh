#!/bin/bash
# Host: push harness files (and packet, if present) to the guest stage, then re-run provision.sh as root.
set -euo pipefail
W=/Volumes/CorbanuDrive/Corbanu/.codex-work/aggressive-targeted-20261014
cd $W
items="h/agms-launch h/agms-probe h/agms-product h/agms-tui h/tmux.conf h/mediation-ca.pem h/product-config.toml h/fixture.tar h/agms_mediator.py h/expected.json"
[ -d stage/packet ] && extra="-C $W/stage packet" || extra=""
COPYFILE_DISABLE=1 tar --no-xattrs -czf - $items $extra | $W/h/vmssh.sh agent@192.168.64.3 'cd ~/agms/stage && rm -R packet 2>/dev/null; tar -xzf - && echo staged'
NEOPW="$(corbanu vault auth-helper neo-vm)" $W/h/admin.sh $W/h/provision.sh
