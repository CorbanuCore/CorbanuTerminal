# Lossless manager action-input indexing — September 13

PF-80-S01 internal coordination correction. Product heading: **Internal delivery
control — TO BUILD**, “Use sequential sprints per initiative”. This is not a
functional acceptance, sprint completion or recurring-operation claim.

The accepted owner-transition implementation ca0474fe4 remains awaiting receiving
integration. Its Fable review session 01a09c94-af28-77e2-80cb-7341ca9287b5 found no
issues. Native worker Nash returned and was actually closed; the coordinator
records acceptance for integration, not a completed sprint.

Two receiving-manager attempts were retained: 8d744475 stopped before TUI launch
because the private socket path exceeded its limit; a39a20d9 stopped while building
the briefing because repeated frozen inputs exceeded 64 KiB. No integration
worker was dispatched by either attempt. The first claim was explicitly
reconciled; the second requires explicit reconciliation before a new manager.

The correction replaces only exactly matching action inputs with an allocation
reference, guarded by canonical JSON equality and the allocation digest. Changed
historical inputs remain inline. The manager is told how to reconstruct the full
record and must still submit full, strictly validated proposal inputs. Raw claims
and all original evidence remain untouched; limits and authority are unchanged.

Verification: 108 coordinator/manager/native-owner/integration tests passed in
7.755 seconds. Both governance checkers and diff checks pass. The retained failed
packet reconstructs to 62,078 bytes, with 40 original evidence bodies, six input
references and no omissions. This is an offline replay, not a successful fresh
manager invocation.

Fable input-review01 reported the code correct and one P3 documentation-spacing
finding. The three identified spacing lines were corrected; no code changed after
review and no extra unchanged-code review was commissioned. Full structured review,
TMUX logs and original failures remain under private owner-transitions-review.Dp4tSw
and transition-receive.qL5nIN. This review is additional to, not a reset of, earlier
bootstrap review history. The owner accepts internal-only functional N/A for this
packet projection; isolated combined dashboard/Slack acceptance remains required.

## Subsequent actual attempts — not a launch pass

After this correction was committed as 5056ba764, receiving-manager attempts
ea397085 and a75bf3e7 also stopped before inference at the briefing-size gate.
The former packet measured 65,805 bytes in a read-only diagnostic; the runtime
limit was not changed. New pending reconciliation/allocation events had consumed
the recovered room. Shortening redundant owner prose was insufficient once the
next failure record was included. Every attempt retains its original claim and
hold; no receiving worker was dispatched and ca0474fe4 is still not integrated.

Stop prompt-trimming/relaunch attempts. The required next owner correction is
bounded pending-event selection: include and consume only the selected durable
batch, retain unselected events for later cycles, preserve all three workstreams
and ordered recent actions, and prove selected evidence is complete. Oversized
single events must remain explicitly held, never silently consumed or truncated.
This is manager-owned implementation, not a missing user approval or a reason to
resume product work. The accepted indexing correction remains useful but does
not establish a reliable unattended manager loop.
