Functional check for accounting; do exactly this and nothing else.
Run each of these shell commands as its own separate tool call, one after another, waiting for each result:
1. sleep 25; echo first; date -u
2. sleep 25; echo second; date -u
3. sleep 25; echo third; date -u
4. sleep 25; echo fourth; date -u
Then reply with the single word DONE.
