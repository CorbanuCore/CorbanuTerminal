Subject: Private inference: should it be paid in PFT or in USDC/USDAI?

Hi Alex,

We've drafted a user-flow spec for private inference in PF Terminal: PR #170, docs/specs/private-inference-user-flow.md.
It came from the Task Node task "Private inference user flow spec for PF Terminal", which asked for the privacy
tiers to be priced and settled in PFT. That runs into the product spec, so I need your call.

How the draft works
- Three tiers: Aggregated, Zero retention, and Verified (comes with a proof the user can check on their own machine).
- Each Corbanu API account gets a second balance, in PFT. Users top it up by sending PFT to a Corbanu deposit address.
  Each private-inference request is charged from that balance.
- The existing USD balance, funded with USDC, keeps paying for everything else. The two balances never convert into
  each other.
- I've decided the chain: PFT would settle on the ledger where Task Node pays PFT today. We can move to the
  PostFiat L1 later.

The conflict
The product spec funds Corbanu API in USDC and lists "USDAI as the preferred stablecoin partner rather than centering
a Corbanu-native token." PFT isn't a Corbanu token. But charging in PFT adds a second money flow and makes a
non-stablecoin the way to pay for a paid feature. The spec also flags stablecoin payments for legal and compliance
review, and taking a volatile token as payment probably adds to that review.

Options
1. PFT only, as drafted. Amend the product spec to allow PFT for the private-inference tiers. This is the most direct
   tie between Post Fiat and the product, but PFT-denominated prices move with the token.
2. USD balance only: USDC now, USDAI if that partnership lands. The product spec stays as is. This is simplest for
   users and for compliance, with one balance and stable prices. PFT plays no role in paying for private inference.
3. Both. Price in USD, and let users pay from either the USD balance or PFT converted at a quoted rate. This is the
   most flexible but the most work, because it needs a price source and a rule for how conversion works.

Two smaller decisions are also yours
- Rates, and whether to show a USD reference next to PFT prices (if PFT is used at all).
- Refunds when a Verified-tier proof fails or isn't available: a full refund, or a refund down to the Zero-retention
  price.

Nothing on the Terminal side is blocked while you decide. Receipts, the proof checker and the tier picker can be
built first. Only the payment step waits on this.

Thanks,
Travis
