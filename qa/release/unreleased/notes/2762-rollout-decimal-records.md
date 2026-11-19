## Fixes

Rollout records holding a decimal number, such as a token-count record with rate-limit percentages, now read back correctly. Before, several rollout readers rejected them: a resumed thread could restore an older token total, the next record appended after such a line could reuse an ordinal, and thread listing, search and `corbanu doctor` skipped those lines (#2655, #2762).
