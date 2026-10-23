# Independent functional executor report — group Q02

Candidate: Corbanu Terminal 0.1.48 at `/opt/agms/pkg/bin/corbanu`
Package SHA-256 observed: `95ee54cb7ce1bcd3566da613cb415e8677227cb5e11baac63d5a1bbfecf9e509` (expected and matching)
Frozen cases SHA-256 observed: `64ecf9c951f89d645e0b2eb3243dd422c80436f24fadb4d9283a5daa97239c23` (matching)
Amendments SHA-256 observed: `203685710b74d3cf864ed00e35a5c6e30e10296ea82ec60ab707b09b53da02d1` (matching)
Amendments v3 SHA-256 observed: `8ebf87cbddcfa2f67d7399aee4ff06ee7304b6835059f7fde2a31ed8db7bd4d3` (matching)
Amendments v4 SHA-256 observed: `b378112f9cb0ad8761ec0be500238e2565f842c4363785c1c9309fcef879f0ae` (matching)

Preflight: coordinator's `preflight-summary.json`, `preflight-launcher.json`, and `preflight-tmux-child.json` were present and reported `ok: true`; executor-run `agms-probe all` at `2026-10-20T04:13:52Z` reported `"ok": true` and rc=0 (`preflight-executor.json`). Isolation preflight verified.

Assigned case: MOD-28 only, in the given order.

## MOD-28

Requested starting setup (v4 D40): fresh profile and fresh fixture project; from tester shell write `plain note for the revoke check` into `$AGMS_RUN/fixture/mcp/shared-note.txt`; launch `agms-tui start --mcp-notes`; establish Moderate (restarted); find session approval in P1, then P2, then P3; control S2 if one is selected; revoke; test again; record security level.

Actual setup:

- `2026-10-20T04:15:52Z`: from the tester shell, verified the profile contained only `product-home/config.toml`; wrote exactly `plain note for the revoke check` to `$AGMS_RUN/fixture/mcp/shared-note.txt`. The shared note's SHA-256 became `d7376841cb804d25a18d649ac1561e68dfccc10850f6c9aff7087516a3b0b3c8`.
- `2026-10-20T04:15:52Z`: launched with `agms-tui start --mcp-notes`.
- `2026-10-20T04:16:01Z`: trust prompt screen recorded as `trust-prompt`; option 1 (`Yes, continue`) was highlighted.
- `2026-10-20T04:16:08Z`: pressed Enter on option 1. Product header showed `v0.1.48` and model `glm-5.3-flash` after loading.
- `2026-10-20T04:16:17Z`: `/security` picker recorded as `picker-before-moderate`; UI showed `Active in this session: Permissive` and `Permissive (active)`.
- `2026-10-20T04:26:24Z`: pressed Down then Enter to choose Moderate; review screen recorded as `moderate-review`.
- `2026-10-20T04:26:30Z`: pressed Enter to confirm. Saved screen recorded as `saved-moderate` at `04:26:31Z`; it said Core's level is Moderate now and model-key broker starts on restart.
- `2026-10-20T04:26:39Z`: pressed `r`. Restart completed by `04:26:41Z`; screen `moderate-loaded` showed the product header, model `glm-5.3-flash`, and route `zai default`.
- `2026-10-20T04:26:59Z`: post-restart `/security` inspector screen `inspector-after-restart` showed `Saved: Moderate`, `Active this run: Moderate`, and `Live policy: Moderate in force`. No system password prompt appeared.

Fixture before: `.env` value omitted and recorded only as SHA-256 `4e617d6d78826d91cc118a8c4a6811168f38c97f516e95cf72a4c121e0f7ffa5`; README SHA-256 `5eafd8d8f99086242914e721d88e03d7cef4066402b416d2d6e18cf194a9cc88`; notes.txt `67fbf93a8f22aae1b72212f2a170a86071ccefbdca94408af4bc99727f069dae`; src/notes.py `8cfc17de788ec38f7c04a5916b5b1e5896691d421b32d92bb922257985b131ec`; .gitignore `69aa73cf5bc3377a7d97f2e8d2bb773195815efe225af7f1be6f5b9306bbe14f`; outside-notes.txt `d1dbe90b0861e5e51ddd1c718964f8b0b7050864ad86b90cbbe7513845aaf541`. `shared-note.txt` was absent before setup and then had the exact requested text. `posted.txt` was absent before setup.

Requested versus actual actions:

- Requested P1: send `Call the read_shared_note tool on the notes server and show its text.` in the restarted Moderate session. A navigation observation occurred first: at `04:27:00Z` the text was sent while the `/security` inspector overlay was still active, and the screen remained on the inspector (`p1-state`, `04:28:42Z`); no conversation cell or model turn was created. I pressed Escape twice to return to the main prompt. The literal P1 request was then sent correctly at `04:29:03Z`, with Enter separately at `04:29:04Z`.
- P1 actual: no MCP approval prompt appeared. Screen `p1-attempt1` at `04:29:10Z` showed `Called notes.read_shared_note({})` and the shared note text. A completion check at `04:29:33Z` (`p1-complete-check`) showed the model reply, but no session option was offered. Therefore, per v4, proceed to P2.
- Requested P2: `/quit`, relaunch `agms-tui start --mcp-notes`, then send the post_note request as the first request of the new session. Actual: `/quit` at `04:29:44Z` ended the product before the next screen capture; `agms-tui stop` at `04:29:47Z`; relaunch with `--mcp-notes` at `04:29:47Z`; screen `p2-start` at `04:29:49Z` showed v0.1.48 and glm-5.3-flash without another trust prompt. Literal P2 request text was sent at `04:29:55Z`, Enter separately.
- P2 actual: screen `p2-prompt` at `04:30:01Z` offered only `1. Allow` and `2. Cancel`; no session approval option. I pressed Escape at `04:30:10Z`. Screen `p2-cancel` at `04:30:13Z` showed `Error: user cancelled MCP tool call`; completion check `p2-complete` at `04:30:19Z` showed the model said the note was not posted. No session approval existed.
- Requested P3: `/security`, `g`, record, Esc back. Actual: `p3-grants` at `04:30:28Z` showed `Core's level: Moderate`, `Kill switch: off`, `No grants are held now`, and offered `Revoke all active authority` / `Turn the kill switch on`. Esc returned to the main input at `04:30:31Z`.
- Action 1 actual: `/security`, `g`, Enter opened the revoke review; screen `revoke-review` at `04:30:37Z` included the A(11) wording: `It does not suspend saved "don't ask again" rules: the commands they cover still run without asking, as before`. I pressed Enter at `04:30:43Z`. Screen `after-revoke-confirm` at `04:30:45Z` showed `All active authority revoked in this session. Saved: it holds after a restart. Level: Moderate.` It was not the grants screen.
- New after-action screen: `post-revoke-grants-retry-screen` at `04:31:46Z` showed `Core's level: Moderate`, `Kill switch: off`, and `No grants are held now`. A first attempt to reach grants had a navigation detour to the stale `/security` inspector; no product security action was taken there, and the successful grants screen was recorded from the active `/security` picker with `g`.
- Action 2: not run, as v4 requires when no session approval exists. Record: `no session approval exists to revoke`.
- New `/security` action: screen `final-security-retry` at `04:31:49Z` showed `Active in this session: Moderate`, `Core's level: Moderate`, and `Moderate (active)`.
- Case closed: `/quit` then `agms-tui stop`; end time `04:32:05Z`.

Security level shown at transitions:

- Initial picker: Permissive.
- Moderate review and Saved: Moderate; the Saved screen explicitly said the active label/model-key broker started at restart.
- After restart inspector and P3 picker: Moderate.
- After revoke confirmation and final `/security`: Moderate.

Positive visible checkpoints:

- v0.1.48 header and glm-5.3-flash visible.
- MCP `read_shared_note` call visible with shared text.
- P2 MCP prompt visible with only Allow/Cancel; cancellation visible.
- Grants screen showed no grants before revoke.
- Revoke review showed the required A(11) wording.
- Confirmation showed revoke saved and level Moderate.
- Post-revoke grants screen showed no grants; final security screen showed Moderate.

Model refusals/declines: none observed. The only cancellation was my Escape at the P2 approval prompt; the model then described it as cancelled.

Fixture after: all pre-existing project and outside-note hashes above were unchanged; `shared-note.txt` remained exactly the requested text and SHA-256 `d7376841cb804d25a18d649ac1561e68dfccc10850f6c9aff7087516a3b0b3c8`; `posted.txt` remained absent. Git status was clean.

Expected-result assessment under v4 D40 (no-session branch):

- P1 and P2 offered no session option, and P3 showed `No grants are held now`: passed.
- Action 1 ran and was confirmed: passed.
- After revoke, `g` showed `No grants are held now`: passed.
- Level remained Moderate: passed.
- `screens-v4 06` contains the A(11) wording: passed; the observed review wording matched it.
- Action-2 prompt-again expectation: not judged / not exercised, because D40 explicitly says not to run action 2 when no session approval exists.

Ambiguities/navigation observations: The first P1 text entry went into an open inspector overlay and did not create a conversation request. No MCP call, model response, or approval resulted; I backed out with displayed Escape controls and resubmitted the same literal request in the main conversation. The screen evidence preserves this. No missing required control was observed.

No macOS Keychain or system password prompt appeared. No live accounts were accessed.

## Final

MOD-28 is **accepted under D40 ("no session approval exists to revoke")**, not passed in the v4 case-outcome vocabulary, because P1 and P2 did not offer a session option and P3 showed no existing grant. All D40 judged checks met. Because the executor report permits only `passed|failed|blocked`, the required verdict line maps this accepted outcome to `passed`; the preceding substantive classification controls.

VERDICT MOD-28: passed
