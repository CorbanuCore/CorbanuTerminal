#!/bin/bash
# C6 real-ledger copies (read-only source, consistent SQLite backup into disposable homes; the sources are never opened
# by a Corbanu binary). acct64/old-A has a retention checkpoint of 2026-12-21 (faked-clock experiment) and reads as
# "Unavailable" (clock-behind, #287); acct63/home-old and the runtime home have sane checkpoints.
S=<scratch>; R=<corbanu-root>/.codex-work
for pair in "h-c6real:acct64/old-A" "h-c6real-a63:acct63/home-old" "h-c6real-live:corbanu-terminal/home"; do
  h=${pair%%:*}; src=${pair#*:}; mkdir -p $S/homes/$h
  printf 'model = "glm-5.2"\nmodel_provider = "zai"\n' > $S/homes/$h/config.toml
  sqlite3 -readonly $R/$src/pfterminal_state_5.sqlite ".backup $S/homes/$h/pfterminal_state_5.sqlite"
done
# Linux comparison: the a63 copy opened by the S05 build and by the S03-accepted build (63ea3d0cbd) in separate homes.
