The spec is written and up for review, and Task Node has marked the task **Rewarded** at 2.25 PFT (2.5 was offered; Task Node didn't give a reason for the difference).

**Deliverable:** `docs/specs/private-inference-user-flow.md` in PR https://github.com/CorbanuCore/CorbanuTerminal/pull/170. It's on branch `docs/private-inference-user-flow-spec`, off `origin/main` at `6bd7fe8413`, latest commit `f7e9fb6964`. The PR is open and mergeable; I didn't merge it.

**Criteria**

| Task criterion | Met | Where |
|---|---|---|
| Request flow: what triggers it, which tier options appear, what comes back | ✅ | A.1 (four triggers and how the minimum tier is worked out), A.2 (screens), A.3 (normal path), A.4 (failure cases) |
| Three tiers (L1 aggregator, L2 zero retention, L3 verified) shown, priced and settled in PFT | ✅ | §1 and §1.1, B.1 to B.5 (price list, charge formula, deposits, hold-then-charge per request, refunds, withdrawal) |
| What proof or hash comes with each response, and how a user checks it on a CPU | ✅ | C.1 to C.7 (signed receipt, separate L3 proof, eight CPU checks, what each check does and doesn't prove) |
| One document with separate user, payment and verification sections | ✅ | Sections A, B and C |
| Specific enough to start building | ✅ | JSON formats, proposed gateway endpoints, which Terminal files change, 15 acceptance tests, milestones M0–M4 |
| Grounded in what ships today and in the PF-65 research docs | ✅ | §0 lists `/providers`, `/model`, Ambient, Corbanu API (which replaced Corbanu Plan), `/gpu` Vast.ai/RunPod, `/vault` and `/tasknode balance`, with code paths, plus links to the PF-65 plan and sprints |

**Review:** I couldn't find a spawn tool, so I used a fresh read-only `corbanu exec` agent as the independent reviewer instead.
- **Round 1:** failed two criteria and raised 9 findings. I fixed them in `36ae77ebe7`.
- **Round 2:** rated all criteria PASS but left one finding partly fixed and raised 3 new ones. I fixed those in `f7e9fb6964`.

The repo's plan and sprint checks pass, all links resolve, and no secrets are in the diff.

**Evidence receipts**
- **Initial evidence:** `task_evt_4e144110-eae5-4fa9-8683-ee884407c145`, submitted once after confirming the task accepted evidence. The supported command has no option for a fixed idempotency key; it makes its own. So I checked state first and didn't retry.
- **Verification response:** `task_evt_d04d92f3-2be4-4603-8a4d-be08fd8efa10`. The reviewer asked for the tier tables and the billing example pasted exactly; I sent them unchanged from the commit and rechecked the arithmetic.
- **Reward:** tx `6075F5DD81161E15ED333A275EC7B13642626D86B04B6B71CF9954CAD571965B`.
- The submitted texts are in `.codex-work/workers-20261005/tasknode-96e88f7-{evidence,verification}.md`.

**Task-map record:** separate PR https://github.com/CorbanuCore/CorbanuTerminal/pull/172, a single commit on a new branch off `origin/main`. I didn't edit or stage anything in the management worktree.

**Still open (Travis):**
1. **Review and merge PRs #170 and #172.**
2. **Settlement chain:** choose where PFT deposits settle — the ledger where Task Node pays PFT today (the spec's default) or the PostFiat L1.
3. **Product spec conflict:** with Alex, decide whether to amend the product spec. It currently funds Corbanu API in USDC and names USDAI as the preferred stablecoin, so settling in PFT changes a money flow.
4. **Spec contents:** all prices in the spec are placeholders, and the refund policy is still undecided.
5. **Dependencies:** the verified tier only works if Ambient returns a proof with each response, which I couldn't confirm. Words like "private" also stay provisional until the PF-65 threat model is done.