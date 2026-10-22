# First-Task Wallet-Linkage Onboarding

Status: Proposed; documentation-only decision for maintainer review
Scope: First-task request wording and verification of an existing Task Node
account-to-wallet association; no program code or account-linking changes
Source baseline: `c9f358c0a7ece3338d751ad2c3ecf1e091de2b8b`

## 1. Problem and decision

[Project-site issue #5](https://github.com/postfiatorg/postfiatorg.github.io/issues/5)
records a newcomer who supplied an address and a screenshot but could not
identify the requested location or understand what proved account linkage.
The request referred to a profile or wallet settings without a precise route.

Decide which address a first task requires, exactly where the contributor must
find it, and what evidence establishes the association with their account.

Recommendation: require the full Task Node-linked wallet address from the
`wallet:` row of `/tasknode status`, together with the account and GitHub rows
from that same status response. Verify against the authenticated Task Node
account record. Do not use the local Solana wallet as a substitute.

This verifies an existing account association. It does not independently prove
possession of a private key, authorize a payment, or create a new wallet link.

## 2. Verified baseline

All terminal commands, screen names, labels, and source paths described as
existing in this document are pinned to commit
`c9f358c0a7ece3338d751ad2c3ecf1e091de2b8b` (abbreviated `c9f358c0` below).
Proposed request and verifier wording is a recommendation, not shipped UI.
No Task Node web-profile navigation was verified or is prescribed here.

| Surface at `c9f358c0` | Location and exact behavior | Source at `c9f358c0` |
| --- | --- | --- |
| `/tasknode status` | Enter in the terminal composer, then press Enter. The response appears in conversation history under `Task Node status`, with rows in order: `profile:`, `account:`, `github:`, `wallet:`, `tasks:`, `actions:`. | `codex-rs/tui/src/chatwidget/tasknode_menu.rs`, `open_tasknode_status`, `handle_tasknode_status_result`, `tasknode_status_lines` |
| Linked address | The `wallet:` row uses the complete server-provided address when `wallet.linked` is true. When false, it displays `not linked`. A true flag with a missing address can render an empty value; this must not pass verification. | Same file, `tasknode_status_lines`, at `c9f358c0` |
| Account context | `profile:` is the local profile name or `default`; `account:` is the Task Node account ID; `github:` is its username. A local profile name alone is not account identity. | Same file, `handle_tasknode_status_result` and `tasknode_status_lines`, at `c9f358c0` |
| `/tasknode` | The menu is titled `Task Node`, shows `Origin:`, an optional `Corbanu profile:`, and linked-account information. It helps check the intended session but is not the address proof surface. | Same file, `tasknode_menu_params`, at `c9f358c0` |
| `/tasknode link` | Connects the selected terminal profile to a Task Node account; rerun `/tasknode status` afterward. This does not document a procedure for adding a missing reward wallet to the server account. | `docs/features/tasknode.md`, "Link an account", at `c9f358c0` |
| `/wallet` | The `Wallet` menu is for a local Solana wallet. Its header abbreviates the address. `Receive` has the full address as its description; choosing it inserts `Solana receive address: <address>` into history. This is not proof of Task Node account linkage. | `codex-rs/tui/src/chatwidget/wallet_menu.rs`, `wallet_params`, `wallet_items`, `short_address`, at `c9f358c0` |

Product context at `c9f358c0`: `docs/corbanu-product-spec.md`, heading
"Shipping MVP - LIVE" (the source heading uses an em dash), row "Task Node and
identity": "Tasks, evidence, verification, rewards, balances, chat, context,
linked identity". The numbered root-spec structure follows
`ACTION-TURN-COMPLETION-SPEC.md` at `c9f358c0`.

## 3. Exact first-task request

Use the following proposed text for the existing `c9f358c0` terminal surface.
The task issuer must include the expected GitHub username from the assigned
account instead of leaving an unresolved placeholder.

> Confirm the wallet already linked to your Task Node account for GitHub
> `<expected GitHub username>`.
>
> In the Corbanu Terminal profile where you accepted this task, type
> `/tasknode status` and press Enter. Wait for the `Task Node status` response
> in the conversation history above the input box. In that response, find
> `wallet:` immediately below `github:`. Copy the entire address after
> `wallet:`, preserving every character and its capitalization.
>
> Reply to this task with the template below, or attach a readable screenshot
> showing `Task Node status` and the `profile:`, `account:`, `github:`, and
> `wallet:` rows together. Both formats are accepted. Use the latest response.
>
> Do not use the address from `/wallet`: that is your local Solana wallet.
> Do not send a seed phrase, private key, passcode, API key, or session token.
> No transfer, payment, wallet unlock, or signing operation is required.

```text
Location: Corbanu Terminal > /tasknode status > Task Node status > wallet:
profile: <value from this response>
account: <value from this response>
github: <value from this response>
wallet: <full value from this response>
```

Proposed recovery copy, referring to the `c9f358c0` surfaces:

> If `wallet:` says `not linked`, is blank, or you see an error instead of a
> status response, report that result here; do not substitute another address.
> If the terminal session is not linked, use `/tasknode link`, complete the
> account login, then run `/tasknode status` again. If the GitHub account is
> wrong, open the intended Corbanu profile or relink to the intended account
> and rerun status. If the correct account still has no wallet, report
> "Task Node wallet not linked" so the task reviewer can provide the current
> account-wallet linking procedure. Do not create a local Solana wallet to
> resolve a missing Task Node wallet.

## 4. Acceptance criteria and verification

These are proposed acceptance rules for the `c9f358c0` surfaces above.

1. The request names the Task Node-linked wallet, expected GitHub identity,
   exact command, response heading, row label, and both allowed evidence
   formats. It never asks only for a "displayed address" or unspecified
   "Profile or Wallet Settings".
2. The contributor can locate the full address without inspecting source,
   using a block explorer, paying, unlocking a wallet, or asking what the
   request means. Wrapped display lines must preserve the entire address;
   ellipses, a cropped image, a prefix/suffix, or a balance alone cannot pass.
3. Text evidence includes the location and the four identity/context rows in
   the template. Screenshot evidence shows the same rows and heading legibly
   in one response. Unrelated conversation, balances, and task counts may be
   cropped. No secrets are requested or included.
4. The verifier resolves the account that owns the accepted task using its
   authenticated Task Node record, not an account ID supplied only by the
   contributor. It checks that record's account ID and GitHub identity against
   the evidence. `profile:` records local context but is not authoritative.
5. At verification time the authoritative record must report a linked wallet
   with a nonempty address, and its full address must equal the submitted
   address character for character. Remove only surrounding whitespace and
   unambiguous visual line wrapping; do not lowercase, abbreviate, or repair
   characters. Reject placeholders and `not linked` as address evidence.
6. A screenshot or copied text is supporting evidence, not independent proof
   of key ownership. If the verifier cannot read the authoritative association,
   classify the result as pending verification, not verified or unlinked.
7. A changed wallet, account mismatch, incomplete address, or stale response
   receives a specific correction request naming the failing row and asking
   for fresh `/tasknode status` evidence. A missing link receives the recovery
   instructions above; it must not be treated as a completed onboarding check.
8. Only when all checks pass, return: "Verified: the full wallet address in
   your evidence matches the wallet linked to the Task Node account that owns
   this task." Record the task ID, account ID, evidence reference, checked
   association and verification time in the task's verification record.

## 5. Resolution and rationale

Adopt the request template and verification contract together. The existing
`c9f358c0` status response exposes the association and identity in one place,
so a wording correction can use an existing surface without a terminal code
change. The server's association determines the wallet's meaning; the visual
shape of an address and ownership of a local wallet do not.

Accepting text and screenshots equally avoids repeating the issue's failed
screenshot loop. Specifying exactly what is missing makes recovery actionable.
Server comparison prevents a plausible copied address or edited screenshot
from being sufficient on its own.

This proposal does not redesign account registration, add a cryptographic
challenge, change rewards, or invent a web-profile label. The maintainer must
approve and arrange adoption of the task/verifier wording; merging this spec
alone does not deploy it. Any later code or authorization change follows the
repository's product decision process separately.

## 6. Validation procedure

Use `c9f358c0a7ece3338d751ad2c3ecf1e091de2b8b` for baseline reproduction.
For a later candidate, record its full source SHA and binary version, recheck
the labels and routes, and update the request if they differ.

1. Confirm `git rev-parse HEAD`. Review the functions and documents in Section
   2 at that commit. Confirm the spec PR adds only this root document and
   `git diff --check` passes.
2. Launch the terminal with a test account that already has a linked wallet.
   Type `/tasknode status`, send Enter separately, and observe the response.
   Record the binary version, source provenance, local profile, exact labels,
   whether the full address is readable, and the observed outcome. Keep test
   evidence free of secrets.
3. Give a newcomer only the proposed request, with the expected GitHub
   username filled in. Have them independently produce text evidence and then
   screenshot evidence. Both must identify the location and complete address
   without clarification; otherwise revise the wording and repeat.
4. With controlled test accounts/fixtures, exercise the cases below. Do not
   unlink or change a real contributor's wallet solely to run a test.

| Case | Required result |
| --- | --- |
| Correct account, linked full address, text template | Verified after authoritative comparison |
| Same account and address, readable screenshot | Same verified result |
| Narrow terminal wraps the address | All characters recoverable; widen or recapture if ambiguous |
| Cropped/ellipsized address or one changed character | Correction identifies incomplete or mismatching `wallet:` |
| Local `/wallet` address supplied | Correction points to `/tasknode status`; no verification |
| Different GitHub account or Task Node account ID | Correction identifies the account mismatch |
| Different named local profile | Evaluate the actual account association, not the profile name alone |
| `not linked`, blank address, or linked flag with no address | Not verified; missing-link recovery |
| Unlinked terminal session | `/tasknode link`, then fresh status evidence |
| Status request fails or verifier cannot access account state | Pending; retry when service access returns |
| Wallet association changes after evidence capture | Fresh response required; stale evidence cannot pass |

5. Review both initial and correction wording for requests for secrets,
   signing, payments, and unsupported web navigation. None are permitted.
6. Record case outcomes and newcomer feedback before claiming the recommended
   onboarding flow is validated. A source review alone is not a usability test.

### Evidence collected for this proposal

On 2026-09-29, the checkout was confirmed at the full baseline SHA above and
the Section 2 source paths were inspected. Installed Corbanu Terminal 0.1.48
was opened in a real PTY; `/tasknode status` followed by Enter produced
`Task Node status`, `profile: default`, and nonempty `account:`, `github:`, and
full `wallet:` rows matching the active Task Node session. Account values are
intentionally omitted from this public document.

The installed executable's source SHA was not established, so this is live
corroboration of the labels, not a claim that the executable was built from
`c9f358c0`. Exact-commit references above are source-verified. No fresh-newcomer
study or negative-case execution is claimed; those remain the validation work
for adoption described in this section.
