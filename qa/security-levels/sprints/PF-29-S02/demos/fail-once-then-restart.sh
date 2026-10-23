#!/bin/bash
# Demo wrapper: the first run stops the next credential migration at the
# given point (debug builds only, as if Corbanu crashed there); the restart
# runs without it so recovery can finish the migration.
REAL="${PF29_CANDIDATE:?set PF29_CANDIDATE to the candidate corbanu binary}"
CORBANU_TEST_MIGRATION_FAIL_AT="${PF29_FAIL_AT:-rewritten}" "$REAL" "$@"
printf '\n[demo] restarting Corbanu Terminal with the same profile, without the injected failure...\n'
sleep 1
exec "$REAL" "$@"
