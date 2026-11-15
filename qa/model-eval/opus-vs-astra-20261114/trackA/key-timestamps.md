# When the frozen keys were written (evidence)

The headers of trackA/key.md and grade/blind-key.md say "~11:40Z". That was a wrong estimate by bench-1114. The headers are left unchanged so the recorded sha256 values stay valid. File birth times (UTC, from stat on the experiment folder):

- 2026-11-14T11:09:19Z  trackA/key.md
- 2026-11-14T11:09:19Z  trackA/key.sha256
- 2026-11-14T11:09:19Z  blind-key.md
- 2026-11-14T11:09:19Z  blind-key.sha256
- 2026-11-14T11:11:24Z  runs/trackA/a01-opus.log
- 2026-11-14T11:11:24Z  runs/trackA/a01-astra.log

So both keys were written at 11:09:19Z, two minutes before the first arm started (11:11:24Z). Current sha256: key.md ccc920d4c45e63f0092607d81115e73ed04613d73d7b97aef03dff72c0304664, blind-key.md 3b9f9eafd32023a4c5c6b182b993561a934fb5792c104220a584df87112d3372, matching the recorded .sha256 files.
