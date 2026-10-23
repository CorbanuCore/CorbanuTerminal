#!/bin/sh
echo "$(date +%s) account=${CORBANU_PROVIDER_ACCOUNT-<unset>}" >> /Volumes/CorbanuDrive/Corbanu/tmp/pf84acc-scratch/mock/authcmd.log
printf 'cmdtok-%s\n' "${CORBANU_PROVIDER_ACCOUNT:-default}"
