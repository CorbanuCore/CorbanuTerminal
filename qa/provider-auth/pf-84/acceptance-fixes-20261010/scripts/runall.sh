#!/bin/bash
S=<scratch>
CLAUDE_CODE_OAUTH_TOKEN="$(env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME /Users/Neo/.local/bin/corbanu vault auth-helper claude-plan-test-token)" $S/claude414.sh > $S/claude414.out 2>&1
$S/tok415.sh > $S/tok415.out 2>&1; $S/dbg418.sh > $S/dbg418.out 2>&1; $S/exec419.sh > $S/exec419.out 2>&1; $S/mock419.sh > $S/mock419.out 2>&1; echo ALLDONE >> $S/mock419.out
