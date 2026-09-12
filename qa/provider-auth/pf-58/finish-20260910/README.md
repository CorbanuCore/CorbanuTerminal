# PF-58 closeout: expanded functional journeys

This packet supersedes the earlier `blind-repairs-20260910` packet for the next
candidate. Earlier failures and package receipts remain historical, not passes
for a newer binary. All 26 frozen cases remain in `dispositions.md`.

## Authority and scope

The user authorized steps 1–5 (expanded journeys, trusted Apps recovery, fresh
live setup, Applications shortcut, commit/push) and **one** final review, pass 9.
Main is not to receive this branch until human acceptance. No credential ACL,
system trust, billing fallback or automatic environment-to-vault migration is
authorized. The final review will use the frozen expectations, not a new design.

Product authority: active `unified-provider-auth.md`, PF-58; product specification
**Shipping MVP — LIVE**, “Encrypted `/vault`, masked entry, metadata-only inspection,
and operational credential use without placing raw values in chat.”

## Additional defects caught before handoff

1. Custom providers sharing a model name were indistinguishable in the Other
   picker tab. Rows now include the exact provider ID.
2. Selecting a custom model without an effort submenu could use the old provider.
   The direct-selection path now retains the explicitly selected provider ID for
   both runtime selection and persistence. Tests inspect the actual endpoint and
   credential route, not only labels.

The new Rust snapshot/action test covers both defects. The updated existing
same-slug snapshot was visually inspected. Prior repairs retain captured-provider
warnings, configured-versus-verified wording and hidden-progress Escape handling.

## Test methods and boundaries

- Expanded journeys use actual TMUX keys, isolated profiles, synthetic credentials,
  a deterministic Responses service, exact candidate hashes, and request records.
  They exercise cancel/selection, high/low/no-effort requests, two restarts,
  disabled-provider persistence, current-provider replacement, 80×24/40×18/140×44
  resizing, actual PTY attach/detach, late responses, 403/429/500 versus 401, and
  replacement of a nonselected credential followed by requests on both routes.
- Trusted Apps testing uses the canonical ChatGPT origin with a disposable,
  process-local TLS CA/proxy. The proxy never forwards traffic; only synthetic
  requests trust that CA. No host file, DNS, system trust or production origin guard
  is modified. An expired Apps account produces the named warning; another model
  works; keyboard account recovery reconnects Apps; Code Mode invokes an actual
  harmless MCP tool and receives its result in the same process.
- Live OpenAI device login was completed by the user. A separate Claude managed
  token was entered from the designated private file, without printing it or
  treating it as an Anthropic API key. Both replied without restart, then the
  profile survived restart. These initial live runs used the predecessor package
  `86573158…`; final-package retained-profile runs are separate evidence.
- Synthetic runs may trace; live credential runs do not globally trace. Only
  sanitized visible panes and metadata receipts belong in this packet; credential
  files, login challenges and private live histories do not.

## Latest-main reconciliation

Incoming `295aed26e53b17f919f7199ae1c9748b1b1250ba` is documentation/governance
only. Three ledger conflicts were resolved preserving this branch's PF-58 work
and completed security sprints. The unallocated PF-76 draft moves from duplicate
order 78 to free order 83; completed PF-20-S03 retains its historical order.
The combined P0 ledger contains 52 current and 31 archived sprints. Sprint and
plan validators and all 13 code-blind checker regressions pass.

## Not a blanket acceptance

Fresh live credentials for every API vendor have not been supplied. Native
Applications consent/deny/cancel trials still require the user. Environment-owned
credentials require their owning environment to change; no silent credential
migration is introduced. A successful targeted automated journey is not a waiver
of a larger frozen case. The human guide keeps these prerequisites yellow and
does not check off human acceptance automatically.

Final candidate hashes, platform runs, shortcut state and review verdict are
recorded in the adjacent manifests and receipts. The branch is to be pushed for
backup, not merged or released by this closeout.
