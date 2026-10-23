# Designer session access record

Installed `corbanu exec` (claude-opus-5-5-plan, high, `-s read-only`), fresh context, working directory = a copy of `packet/` outside the source checkout. Isolation was instruction-only (read-only sandbox; no tool restriction to the packet). Commands the agent ran:

- `/bin/bash -lc 'cat intent.md; for f in screens/*.txt; do echo "=== $f"; cat "$f"; done'`
