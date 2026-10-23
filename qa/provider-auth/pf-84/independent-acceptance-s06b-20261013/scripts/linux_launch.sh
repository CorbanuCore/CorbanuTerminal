#!/usr/bin/env bash
# Mac side of REG-06: copy linux_glitch.sh to glitch and run it with the real Z.AI key on its stdin.
D=$(cd "$(dirname "$0")" && pwd); source "$D/env.sh"
scp -q "$D/linux_glitch.sh" glitch:corbanu-rtx/s06b-qa-run.sh
key provider/zai_api_key sh -c 'printf %s "$SECRET" | ssh glitch bash corbanu-rtx/s06b-qa-run.sh' > "$EV/captures/linux-reg06.txt" 2>&1
echo done
