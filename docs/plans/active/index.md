# Active plans

This directory contains the product initiatives authorized for implementation.
At most three plan files may declare `status: active`. Each reserves one sprint,
or up to three when it declares `parallel_lanes` (P0 security since 2026-10-06).

Current active initiatives:

- [1. PF-13 security and protected credentials](p0-security-levels.md)
- [2. Accounting](portfolio-agent-cost-accounting.md)
- [3. Task Node integration and beta planning](initiative-delivery-control.md)

Run `python3 docs/plans/check.py` before adding, moving, or closing a plan.
