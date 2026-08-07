# Run 02 — fresh wallet, funding, Basic purchase, and restart persistence

- Stories: US-1, US-2, US-3, US-7, US-8
- Started: 2026-07-19T04:51:36Z
- Terminal: tmux `pft_wallet_fresh_basic`, 94×57
- Isolated home: `/tmp/pft-wallet-qual-run2-home`
- Public wallet: `J6xbzsav5CixZP3ndbpxUxuv28y2kstXHrEZegDNgNhk`
- Funding transaction: `43FbNcw98rbhAmzsn9c1gmQhcxBUEqvhs48H4dUvezQLrmKvsiUdkPHNtg63v5dk5fWMq2MAZw3hnZYE42StEXmz`
- Purchase transaction: `mjA2KW7bC9xx9CqTumJJsTpvvMgqm3sk74u66QEFCtQBH4RY32UzHEYWiCaw7SUz4nXPH7yTA2xMFfu7ZS9W1iG`

## Steps and evidence

1. Created a new wallet through the actual TUI with a masked passcode and
   confirmation. Recovery material remained confined to the secure view.
2. Funded it with exactly 20 USDC and 0.010000 SOL from the authorized operator
   wallet. The finalized on-chain transfer and refreshed TUI balances agreed.
3. Opened the plan catalog. Starter and Basic were selectable; Power and Pro
   were disabled with their exact USDC shortfalls.
4. Selected Basic, reviewed the 20-USDC amount, weekly/monthly allowance, and
   period, then confirmed from the unlocked TUI.
5. The receipt showed the full transaction, active Basic period, 20-USDC debit,
   and zero remaining USDC. Direct server account state agreed.
6. Exited and restarted the process against the same isolated home. The wallet,
   active plan, plan credential, and provider selection persisted.
7. Started paid inference and confirmed authoritative usage settlement rather
   than reservation charging.

## Verdict

FAIL for the complete story. Fresh wallet persistence, funding, catalog
affordability, Basic purchase, durable plan credential, provider selection,
and restart passed, but later qualification of recovery backup on a fresh Pro
wallet captured the secure recovery view into driver tool output. The secure
view remained outside the product transcript and persisted state, but the
driver violated the evidence boundary. The Pro wallet and its local credential
were retired, the server key was revoked, and no result from that wallet counts
toward final qualification. See WSEC-001.

The cleanup also exposed WUX-015: repeated acceptance input could submit the
same destructive confirmation twice. `341064f0c` repairs the generalized
selection boundary and makes removal idempotent. Run 2 must be repeated from a
new wallet on the rebuilt exact binary, without capturing any recovery view.

## Clean replay on the final binary

- Started: 2026-07-19T07:23:00Z
- Commit: `341064f0c996c334bddc084473588ca7ee9c1a3a`
- Binary SHA-256: `8ef6087d5079793f14f1b3a37bf9ecfc3c5e067ae7c33fcb7d7e1200aa6942e0`
- Terminal: tmux `pft_wallet_final`, 94×57
- Fresh home: `/tmp/pft-wallet-qual-final-home`
- Public wallet: `4RBjjHAor14bpxwpZgNREQ8MeGpB64sam6rrvchNxSGn`
- Funding transaction: `4X7H3xeQttbmodhH98pjFtkHVRMD7M78kq9AwgZk8VYDzvRx1cW7fqmKiYM4WZKLHuK6j7y8FrT7zNw2ZdJjP6WL`
- Purchase transaction: `2Ebz9CjGDbTZhkiu8JqF9nCGTswSBeC5NzFE2fKwpAbf2hZGz5ab9jp8qCx8NgdZbtADC5MUa9KTAW1nc19vvvoL`

The visible TUI created a new wallet using masked passcode and confirmation
views. The recovery screen was acknowledged without any driver capture.
Funding refreshed from authoritative zero to 0.010000 SOL and 200.00 USDC. The
server catalog offered all four affordable tiers; Pro confirmation stated the
exact debit, allowance, post-payment balance, sponsored fee, and non-recurring
nature. One payment settled, produced one active Pro period and one live
customer key, selected PfTerminal Plan, and displayed the complete durable
receipt. Server rows matched the TUI.

The user then re-exported backup material using a fresh passcode. The secure
view was not captured; after it was acknowledged the transcript contained only
`Recovery backup acknowledged. The secure view was cleared.` A literal scan of
the passphrase found zero matches in the isolated home, evidence tree, process
environments, and tmux buffers. After closing and reopening PfTerminal, the
same wallet and receipt persisted, signing was locked, and the PfTerminal Plan
provider/model selection persisted.

Clean replay verdict: PASS for US-1, US-2, US-3, and US-8. US-7 disconnect,
remove, and recovery remain assigned to Runs 5 and 7. The original failed
attempt remains recorded above and is not erased by this replay.

## Exact-binary recovery replay

- Commit: `830b346a6f09db343c8b84e81a5d4b6a7b62c0ca`
- Binary SHA-256:
  `c8477ac135a3405f75d8e42a3b6117059c3c1864a117dc267c7e5645812d4288`
- Fresh copied home: `/tmp/pft-wallet-qual02-home`
- Public wallet: `4RBjjHAor14bpxwpZgNREQ8MeGpB64sam6rrvchNxSGn`

The exact-binary replay began from a locked wallet and an isolated copy of the
paid account. A one-action unlock issued a replacement plan key, moved the
server's active-key count from three to four, reselected PfTerminal Plan, and
immediately relocked. No payment or chain transaction occurred. Backup
re-export required a fresh passcode; the secure view was acknowledged without
capture and the temporary passphrase had zero matches in the isolated home,
evidence tree, process environments, and tmux buffers.

Exact-binary verdict: PASS for recovery access and backup re-export. Wallet
disconnect and cross-process revocation remain assigned to Runs 5 and 7.

## Renewed same-binary replay — 2026-07-19T13:54Z

- Commit: `ac87f60d732ab6993fab0a6417005f079e3c0581`
- Binary SHA-256: `af69f401957637d46bbf01b123dfd2a2367ec61813ca6175912ca60e02be0246`
- Wallet-daemon SHA-256: `337bf8ff83b444183b5631247544b2411b7f10ea2283ee555010716d8a03f39c`
- Terminal: tmux `pft_wallet_run2`, 94×57
- Fresh home: `/tmp/pft-wallet-run2-renewed-home`
- Public wallet: `2YYwro8tH3LzkwqCyHqZvBZt9KBsQwgtu9E6b1dBhbB5`

Portable recovery material was passed only between owner-only temporary files
and secure TUI inputs; it was never captured or printed. The TUI restored the
wallet and, after the WUX-019 repair, emitted `Restored Solana wallet` rather
than creation copy. The restored address, 0.010000 SOL / 0 USDC balance, Pro
period, and current usage matched the authoritative account. A full process
restart returned the wallet to locked state.

The copied bootstrap credential was disconnected visibly; the paid period and
wallet stayed unchanged and the provider became unavailable. A one-action
unlock then recovered plan access through wallet ownership without payment,
stored the replacement credential, reselected PfTerminal Plan, and relocked.
Backup re-export required the fresh restored-wallet passphrase, opened only the
secure view, and ended with `Recovery backup acknowledged. The secure view was
cleared.`

Literal scans of the recovery material and passphrase found zero matches in the
isolated home, evidence tree, spec, readable process environments, and tmux
buffers. Hardened non-dumpable process environments were unreadable by the
driver; secrets were supplied through PTY input rather than environment
variables. No SOL/USDC moved and no new paid period was created.
The owner-only recovery/passphrase files and copied home were deleted as soon
as the paired proof completed; 143 GB remained free against the 60-GB reserve.

Renewed verdict: **PASS** for Run 2 on the exact binary pair above.
