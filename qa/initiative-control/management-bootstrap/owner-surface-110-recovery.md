# Recovery procedure — owner-surface-110

This supersedes the wording of recovery step 2 in owner-assurance-109.md.
It names existing operations; it does not authorize this worker to operate the
live listener, transport journal or coordinator.

1. Preserve the journal and inspect its hold, pending events, ingress fence,
   actual listener session and epoch. Keep the existing listener owner; do not
   manufacture a session or edit the hold.
2. Inspect outstanding gap/quarantine evidence. Drain pending transport events
   with the existing local operation, from the repository root:

   ```sh
   python scripts/initiative_control/decision_manager.py drain --store <operator-store>
   ```

   Supply one newline-terminated JSON object on stdin containing
   `{"binding": <the exact existing pinned binding object>}`.
   Omit `--live`: this operation is local and does not need credentials, even
   when the existing verification has expired or the journal is held. It drains
   at most ten events per call; inspect the returned `drained` count and the
   journal, repeating the same operation while progress is made. A refusal or
   no progress with remaining events requires manager investigation, not manual
   journal edits. Drain records local intake/disposition; it does not deliver
   an answer to a worker, approve work or clear quarantine. Qualification
   requires no undrained transport events.
3. Collect fresh supported-UI evidence and, for a held/review-required journal,
   the exact gap review: watermark, binding digest, actual ingress count,
   live session identifier, epoch and evidence identifier. Review quarantined
   or unknown history explicitly; do not infer delivery from clearing it.
4. Run the existing owner-controlled
   `python scripts/initiative_control/decision_manager.py qualify --store <operator-store> --live`,
   supplying newline-terminated JSON containing `binding`, `ui_evidence`,
   and the required `gap_review`. Use the established credential mechanism;
   credentials do not belong in that JSON. A concurrent qualifier now refuses
   inside this operation before authentication or a hold write.
5. Verify the persisted review, refreshed verification and cleared hold, then
   refresh the projection and confirm the supervisor observation. If the
   session, ingress or epoch changes, inspect and obtain a new exact review.

The existing disposable test
`ManagerTests.test_expired_binding_allows_registered_local_drain_not_send_or_work`
exercises this exact drain CLI on a held, expired transport and fails if
credentials are requested. New tests demonstrate restoration after catchable
qualification failures. Uncatchable process death (including SIGKILL) or failure
of the cleanup storage write can still require this explicit recovery procedure;
neither is represented as automatically restored.
