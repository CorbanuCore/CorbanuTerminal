# #443: Aggressive protects the session as soon as it is confirmed (2026-10-11)

Travis's decision (2026-10-11): protections apply the moment Aggressive is confirmed, not after the restart. At minimum
the vault refusal and the Corbanu home / protected-path denial must hold in the running session. Anything that only the
restart applies must be named plainly on screen 07, which must not say "this session stays Permissive" while
protections are active (also AGG-37's wording).

**Change.**

- While the session's config or live Core level is Aggressive, exec policy includes the Aggressive vault rule, with
  wrapped-form matching. This covers the window after confirmation and before the restart that loads the rules file.
- Protected-path read denials already followed the live level. The milestone log shows them applied from the first
  command after confirmation.
- The review, screen 07 and the `/security` list now say which protections apply now and which start at the restart.
  They also say that commands already running keep the permissions they started with.

## Live check (tmux, GLM 5.3 Flash)

Setup: a private tmux socket, a disposable home with `CORBANU_TEST_NO_NATIVE_KEYRING=1`, the "Aggressive ships" feature
flags, and the real Z.AI key from the vault helper. Scripts: [launch.sh](scripts/launch.sh),
[case443.sh](scripts/case443.sh). The steps match AGG-04: confirm Aggressive, press Esc on screen 07 without
restarting, ask for `corbanu vault list`, then ask for `head -3 "$CORBANU_HOME/config.toml"`.

Builds (macOS arm64, debug):

- **before**: this branch's base without the #443 change (`origin/main` plus only the unrelated #445 change).
- **after**: this branch at `03f165ac28`.

| Step | Before | After |
| --- | --- | --- |
| Screen 07 | "Until then this session stays Permissive" ([capture](captures/before-screen07.txt)) | "Active now in this session: …" and "Starts when you restart: …" ([capture](captures/after-screen07.txt)) |
| `corbanu vault list` | ran: usage error, exit 2 ([capture](captures/before-vault.txt)) | refused: "Security level Aggressive: agents cannot use the Corbanu vault." ([capture](captures/after-vault.txt)) |
| Corbanu home `config.toml` | "Operation not permitted" ([capture](captures/before-home.txt)) | "Operation not permitted" ([capture](captures/after-home.txt)) |
| `/security` list | "Saved: Aggressive. It takes effect when you restart" ([capture](captures/before-list.txt)) | names Core's protections as active now ([capture](captures/after-list.txt)) |

The review screen after the change: [capture](captures/after-review.txt).

## Not changed here

- **Still at restart:** the Aggressive sandbox, approvals, network, environment stripping, model-key broker and
  child-agent values. They are named on screen 07.
- **Listing the home:** a listing of the Corbanu home still shows file names. Reads of the protected entries are
  denied, before the restart and after it alike.
- **Hidden command:** in both runs the sandbox-denied `head` command never appeared on screen as a command. It ended
  while starting, before its begin event. This is fixed separately (see the PR).
