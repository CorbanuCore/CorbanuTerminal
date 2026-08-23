# Contributing

Corbanu Terminal changes must preserve a coherent product, not merely compile.
Before starting work, read the [Development Mandate](development-mandate.md),
the repository-root `AGENTS.md`, and the relevant nested `AGENTS.md` files.

## Start with product intent

- Identify the exact heading and requirement excerpt in the
  [product specification](corbanu-product-spec.md).
- Classify the change as routine, bounded fix, product initiative, or release.
- Use an active plan for a product initiative. At most two product initiatives
  may be active at once.
- Record the implementation worktree, branch, base commit, owner, and scope.

If the product specification does not authorize the outcome, obtain the
required product decision before implementation.

## Keep changes focused

- Work on one classified outcome at a time.
- Preserve unrelated changes already present in the worktree.
- Fix failed behavior at the responsible boundary instead of hard-coding the
  reported example.
- Add regression coverage for the general failure class.
- Follow the nearest scoped engineering guidance for every changed path.

## Prove the final tree

Run the affected formatting, lint, unit, integration, and snapshot checks after
the last code-changing tool. User-facing interactive behavior also requires a
real Corbanu Terminal TUI run in a PTY with actual keys sent. Smoke tests and
non-interactive `exec` runs are supporting evidence, not substitutes.

Release candidates additionally require live workflows in TensorCash and
Isometric Game, named human acceptance, current user documentation, and the
competitive benchmark gate when due.

## Document only finished behavior

User documentation explains the pain solved and the finished user flow.
Unfinished product work belongs in the product specification or an active plan;
raw release evidence belongs under `qa/release/<version>/`.

Every feature page cites the exact product-spec heading and short requirement
excerpt it implements.

## Pull requests

A reviewable pull request includes:

- the change class and product citation;
- the plan link when required;
- a focused explanation of the user outcome;
- final-tree test results;
- true-TUI and live-repository evidence when required;
- documentation changes; and
- any release blocker that remains.

Do not mark work complete while required evidence is missing.

## Security reports

Do not place credentials, wallet seeds, private keys, protected financial data,
or exploitable details in a public issue. Use the repository's private security
reporting channel and provide the smallest safe reproduction that identifies
the failed boundary.
