# Executor packet as staged read-only at /opt/agms/packet

- `v1/`: used by the original attempts G01-G20 (2026-10-10 23:09Z to 2026-10-11 03:51Z).
- `v2/`: used by the replays (G07b, G08b, G10b, G11b, G12b, G15b, G17b, G17c, G18b) and G21. Same
  `design-cases.md` (SHA-256 `cebdc554…cda4`); prompts add the replay notes from `tools/replays.json` and the
  fixture-v2 `--env` rule.
- The screen text copies (`/opt/agms/packet/screens/*.txt`) are the design packet's `design/packet/screens/*.txt`,
  byte-identical, so they are not duplicated here.
