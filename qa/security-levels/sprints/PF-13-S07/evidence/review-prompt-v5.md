You are an independent security reviewer (Opus 5.5 High). Review the NEW
PF-13-S07 saved-level route matrix evidence (v5) in
qa/security-levels/sprints/PF-13-S07/evidence/.

This is a read-only review. Do not modify any files. Only examine the evidence
folder, focusing on the v5 files:
- route-matrix-saved-levels.sh (the harness)
- route-matrix-v5-aggressive-macos.json + route-matrix-v5-aggressive-macos-results.jsonl
- route-matrix-v5-moderate-macos.json + route-matrix-v5-moderate-macos-results.jsonl
- route-matrix-v5-aggressive-linux.json + route-matrix-v5-aggressive-linux-results.jsonl
- route-matrix-v5-moderate-linux.json + route-matrix-v5-moderate-linux-results.jsonl
- README.md (the v5 section and the updated status line)

Context: This is a follow-up qualification round (round 5). Issue #239 was fixed
in PR #244 (merged): under a saved Aggressive or Moderate security level the
sandbox's protected-path read rules apply from the start of the session, so
known credential locations ($HOME/.ssh, $HOME/.aws, Corbanu home stores) are
unreadable before any untrusted content. Permissive is unchanged; an arbitrary
home file such as $HOME/canary-secret.txt stays readable at every level (only
known credential locations are denied).

Check:
1. Is the saved level genuinely persisted? The harness writes it to the
   disposable home's config.toml [security] section (version=1, level=...).
   Is this the correct persistence path, or is it a transient -c override?
2. Are the four runs (Aggressive macOS, Moderate macOS, Aggressive Linux,
   Moderate Linux) all clean (0 leaks)? Are the candidate SHA-256s and source
   commit recorded correctly in each JSON?
3. Is the files_credential_path route ($HOME/.ssh/id_rsa_fake) actually BLOCKED
   under both saved levels? This is the #239 fix confirmation. Does the stdout
   evidence support "Operation not permitted"?
4. Is the mcp_hook route correctly reported as NOT_CONTAINED (unsandboxed by
   design) rather than as a leak?
5. Does the README honestly represent the v5 evidence? Any discrepancy between
   the README claims and the actual evidence files?
6. Are the fake canaries only (no real secrets) and CORBANU_TEST_NO_NATIVE_KEYRING=1
   on every run?

Give a verdict: APPROVE, APPROVE WITH NITS, or CHANGES REQUIRED.
List any P0/P1 findings. Be concise.
