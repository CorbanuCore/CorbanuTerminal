PF-27-S04 is merged behind the `isolated_credential_broker` flag, which is off by default, so Permissive behaves exactly as before. I did not start PF-27-S02.

**What shipped:** PR #180, merged at `13cf4a2d0c`. All 41 checks passed.
- **Inventory:** all 17 commits on `feat/security-broker-resume-20260911` were already on main as equivalent commits, so I redid nothing and started fresh on `feat/pf27-s04-broker-20261006`.
- **The broker:** with the flag on, Core's network proxy starts a separate broker process (`corbanu --codex-run-as-credential-broker`). It hands the broker the GitHub and OpenAI tokens and keeps only dummy values. Agent commands see only the dummies.
- **Requests:** the broker accepts signed, single-purpose requests (host, method, path) only from the Core process. It adds the real token and makes the HTTPS request itself. Revoking or a broker crash blocks those requests; Core never falls back to adding the raw token.
- **Bug found and fixed:** the demo secret scan showed that a trace log in `core/src/spawn.rs` was writing the child environment, including raw credentials, to the logs. It now logs variable names only.

**Gate evidence** (written up in `qa/security-levels/sprints/PF-27-S04/isolated-broker-20261006/README.md`):
- **Tests:** the focused `pf_27_s01` tests pass: 24 in secret-broker and 2 in core. The affected crates pass 334 tests, including 12 new ones that run the broker as a real separate process. 88 related core tests pass. Formatting and clippy are clean.
- **GLM 5.2 tmux run:**
  - With the flag off, the agent sees the raw token.
  - With it on, the agent sees a dummy and the server still accepts the real token.
  - The agent and the user's own shell both fail to call the broker socket directly.
  - Killing the broker blocks requests, and a new session gets a new broker.
- **Review:** one Opus 5.5 High review, plus a re-check of the fixes. Both returned APPROVE WITH NITS with no P0 or P1 findings. The two P2s are fixed: the broker now blocks debuggers and core dumps, and it starts with the proxy and is never respawned. Both reports are on the PR and in the evidence folder.
- **Videos** (recorded with the new demo script, links in `qa/demos/index/PF-27-S04.md`):
  - [substitution](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s04-pf27-broker-substitution-03e35d2e744f-2026-10-05.mp4)
  - [direct broker call denied](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s04-pf27-broker-direct-call-denied-1a4370992f5a-2026-10-05.mp4)
  - [dead broker fails closed](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s04-pf27-broker-crash-fails-closed-1a4370992f5a-2026-10-05.mp4)
  - The earlier tmux recordings are in `.codex-work/broker-lane-20261006/casts/`. That includes GLM's refusal to save the token to a file, recorded word for word.
- **Docs:** I updated the sprint file. To pass the sprint checker I had to add one worktree entry to the plan's front matter. Main's rewrite had replaced the index row, so I took main's version.

**Still open** (the sprint stays in progress):
- **Model-client auth (needs your call):** Core's own model-provider keys are still used inside Core, and tokens Core inherits from its environment still sit there.
- **PF-27-S02:**
  - The broker runs as the same user without an OS sandbox.
  - A small macOS window remains where a process started at the same moment could inherit the broker's setup pipe.
  - The broker's socket sits in a directory the agent can write to, which allows denial of service only.
- **No revocation trigger:** revocation works but nothing in the product calls it yet. Today a run ends when Core exits; PF-23-S03 can call it.
- **Known limits with the flag on:**
  - A dead broker stays down until the next session.
  - Only common HTTP methods and paths up to 1,024 bytes go through the broker.
  - GitHub Enterprise on a private IP needs `allow_local_binding`.
  - Uploads over 5 minutes are cut off, and a response nobody reads holds a broker slot.
- **Windows:** has no broker. With the flag on, agents get dummies and no token is ever added.

**Decision for you:** should moving Core's own model-client auth into the broker stay in PF-27-S04 or become its own sprint? I recommend its own sprint after PF-27-S02. It touches the model clients rather than the proxy, and S02's OS containment is needed before the broker is a real boundary.

**PF-27-S02:** not started. It is a three-OS containment sprint and too big to rush at the end of this session. One design conflict to settle first: stripping secret-looking variables from agent environments in Moderate and Aggressive would also strip the tokens the broker swaps for dummies. S02 has to exempt those or run after them.