#!/bin/sh
# worker started from inside a Corbanu tool shell: inherits the session's env, sets only CORBANU_HOME
export CORBANU_TEST_NO_NATIVE_KEYRING=1
echo "tool shell inherited: CORBANU_HOME=${CORBANU_HOME-<unset>} CODEX_HOME=${CODEX_HOME-<unset>}"
CORBANU_HOME=/Volumes/CorbanuDrive/Corbanu/tmp/pf84acc-scratch/s01/hB CORBANU_LAUNCHER_WORK_DIR=/Volumes/CorbanuDrive/Corbanu/tmp/pf84acc-scratch/launcher/work CORBANU_LAUNCHER_WORKSPACE_DIR=/Volumes/CorbanuDrive/Corbanu/tmp/pf84acc-scratch/launcher/ws sh /Volumes/CorbanuDrive/Corbanu/worktrees/pf84-indep-accept-20261010/scripts/dev/corbanu-launcher.sh doctor --json 2>&1 | grep -E 'warning|"sqlite home"'
