# Bootstrap publication — September 13, 09:33 UTC

Native Luna Extra High Bernoulli `01a09a1a-855f-7bf1-8a5f-05f149ed0eba`
ran the existing source-sync wrapper once. Parent kept canonical source frozen
and closed the worker after its complete handoff.

- Source `2fc80fa360a45bdc0ad51dfd101ac46c9a92cc83`, clean integration branch.
- Remote/local generation `build-uv2183uj`, collected09:33:07, published09:33:13UTC.
- Export/tree hash `54c2418ce4cbb791ad72e4ca43af17b218e1a700d587bf94ab0435f12d15bcb6`.
- Health HTTP200/ok=true,10 warnings;17/17 reference hashes, including the new
  bounded preflight evidence. Health still explicitly marks the decision feed
  stale and Slack state unknown; serving health does not establish Slack readiness.
- Local/remote Facilities HTTP200 and `X-Corbanu-Control: 1`.
- Publish timer disabled/inactive; read-only web enabled/active; existing tunnel
  PID47015 preserved. No off-Mac HTTPS or recurring-operation claim.

Private export `.codex-work/initiative-control.oGQGyA/export.rKmRCu`, including
`state/source.json`, retains exact source/manifest records. Parent independently
read source.json and the live tunnel health response after worker completion;
they agree on source, generation and timestamps. Source freeze is now released.
