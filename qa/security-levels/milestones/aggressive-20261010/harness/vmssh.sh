#!/bin/bash
exec ssh -i /Volumes/CorbanuDrive/Corbanu/.codex-work/functional-pf83.20260915/keys/pf83-vm_ed25519 -o IdentitiesOnly=yes -o StrictHostKeyChecking=yes -o UserKnownHostsFile=/Volumes/CorbanuDrive/Corbanu/.codex-work/aggressive-ms-20261010/known_hosts -o ConnectTimeout=10 -o BatchMode=yes -o ForwardAgent=no -o ClearAllForwardings=yes "$@"
