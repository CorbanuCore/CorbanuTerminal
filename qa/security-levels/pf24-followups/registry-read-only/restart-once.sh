#!/bin/bash
# Demo wrapper: the account home (debug builds read CORBANU_TEST_ACCOUNT_HOME
# in place of the account database) is the workspace, as when a person works
# in their home folder, so the Aggressive-homes registry sits inside the
# workspace. Starts the candidate twice with the same arguments and profile so
# a video can show a real restart.
REAL="${PF24_CANDIDATE:?set PF24_CANDIDATE to the candidate corbanu binary}"
prev=""
for arg in "$@"; do
  if [ "$prev" = "-C" ]; then WORKSPACE="$arg"; fi
  prev="$arg"
done
export CORBANU_TEST_ACCOUNT_HOME="${WORKSPACE:?the demo passes -C <workspace>}"
"$REAL" "$@"
printf '\n[demo] restarting Corbanu Terminal with the same profile...\n'
sleep 1
exec "$REAL" "$@"
