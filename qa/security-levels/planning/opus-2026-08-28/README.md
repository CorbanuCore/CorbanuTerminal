# Outside architecture review — 2026-08-28

Planning evidence, not runtime qualification. The user selected Opus 5 / Extra
after the Fable attempt was interrupted. The completed review inspected the
63-sprint security planning snapshot and selected pinned OpenClaw source;
Corbanu runtime implementation was not in the packet.

- [Raw review](OPUS_REVIEW.md), preserved byte-for-byte:
  SHA-256 `c3b28a1d71229729d91c55149458d89c7b2beb95a385b5f4c362e69fbbdf6aa3`.
- [Independent assessment](REVIEW_ASSESSMENT.md): accepted, qualified and rejected
  claims. Only link targets were made portable on import; this is the historical
  assessment before the user's subsequent authorization to amend the sprints.
- `SNAPSHOT.json` and `FILES.sha256` identify the original 127-file review packet,
  retained at the local evidence path recorded in the snapshot, not duplicated
  here. Checksums describe that pre-amendment packet, not the current tree.
- [Source review evidence](../openclaw-2026-08-28/README.md) records the upstream
  pin and the limited helper tests. The architecture review added no runtime,
  platform, TUI, core-suite or release qualification.

The raw review overstates several claims. Read the assessment alongside it:
OR-14 is absent, OR-20's capacity exploit is unsupported by the underlying store,
and a classifier-less Moderate or delayed emergency kill is not accepted scope.
