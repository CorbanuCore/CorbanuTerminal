# Active plans

This directory contains the product initiatives authorized for implementation.
At most three plan files may declare `status: active`. Each reserves one sprint,
or up to three when it declares `parallel_lanes` (the security plan since 2026-10-06).

Current active initiatives:

- [1. P1 security hardening](p1-security-hardening.md) (from 2026-10-08; replaced the [closed P0 security plan](../completed/main-2026-10-08-p0-security-levels.md))
- [2. Unified provider onboarding: PF-84 named accounts per provider](unified-provider-auth.md) (from 2026-10-10; took the slot of the [completed accounting plan](../completed/main-2026-10-10-portfolio-agent-cost-accounting.md))
- [3. Task Node integration and beta planning](initiative-delivery-control.md)

Run `python3 docs/plans/check.py` before adding, moving, or closing a plan.
