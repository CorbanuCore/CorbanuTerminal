# 11. Cancel a stream and continue the same session

Ask for several paragraphs. While text is streaming, press `Escape`, then send `Reply with exactly: RECOVERY_OK`.

Pass: streaming stops cleanly, the second response is `RECOVERY_OK`, and the composer remains responsive.

Fail: Corbanu panics, exits, stalls, duplicates output, or displays `OutputTextDelta without active item`.

