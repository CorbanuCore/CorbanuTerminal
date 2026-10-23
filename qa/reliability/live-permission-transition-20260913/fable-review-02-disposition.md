# Fable review02 — v2

Fresh Corbanu/Fable5.1High review of v2 patch d8189e02ead3c905972e12b8ba6782d4d9ed8f06e53e9ec7988443dcd6b98e3b
completed at2026-09-14T07:52:07Z, TMUX pf83-review2-hyferz:corrected.
Helper exit1 because two P3 findings; raw review02 JSON/text retained. Overall
reviewer verdict: patch correct, prior two issues resolved; not a clean result.
Budget3/5 used (original design, reviews01/02), next corrected review04 and
independent evidence review05 remain. No review panel or budget reset.

Parent inspected confirm_permissions and later turn override selection. Accept
the unsupported-server state mutation as an in-scope compatibility bug: inserting
server_permission_threads despite known unsupported endpoint changes routing
although no request was sent. Only mark potential server ownership when dispatch
will occur; preserve uncertainty/no-retry behavior for actually sent requests.
Add native behavioral regression for Unsupported completion and unchanged thread
ownership. This is cycle2, narrowly one already-allocated file, not new API scope.
Manager allowance1750total/750non-test (v2 baseline1649/637) only for this test.

The optimistic transcript duplicate is retained as a nonblocking cosmetic
follow-up, per reviewer severity and integrator discretion. Deferred text is
explicitly reported as unsent and restored; actual unauthorized submission,
payload loss or queue replay is not alleged. No claimed functional case pass
or waiver follows from this disposition. Independent execution must still
report actual transcript behavior. Do not refactor transcript ownership now.

After minimal fix, run affected exact-tree tests, freeze v3 separately and
obtain fresh Fable review. This is the second repair cycle: if new accepted
findings remain afterward, pause and reclassify scope before further edits.
No install/merge/live app changes; native isolation prerequisite remains open.
