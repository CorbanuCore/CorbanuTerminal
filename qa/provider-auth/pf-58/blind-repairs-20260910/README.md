# PF-58 blind-test repairs — September 10

Status: targeted repairs verified and final evidence pass 8 complete; **not a human-ready verdict**.
This record follows the frozen 26-case design and failed evidence review in
`../blind-20260910/`. Historical artifacts and dispositions are not overwritten.

## Product changes

- F12: rejection notifications name the provider captured when the request began,
  including its exact catalog ID. Switching providers cannot redirect the warning;
  superseded credential attempts do not produce new warnings.
- F15/F16: non-OpenAI model status includes the runtime provider ID. Identical model
  labels on different custom routes remain distinguishable. External panes retain
  their own display identity instead of inheriting the parent provider.
- F08/F11: provider management explicitly distinguishes credential presence from
  verified remote access. It does not probe providers, fabricate verification or
  import environment-owned credentials into managed storage.
- Historical handoff Escape failure: interruption depends on a running task, not
  visibility of the progress indicator. Modal, agent-command and Vim precedence
  remain intact; idle Escape is not an interruption.

## Regression changes

- Captured provider identity, stale replacement failures and MCP/account isolation.
- Exact-provider footer and external-pane identity.
- Hidden-indicator Escape interruption, plus the existing interruption regressions.
- Real-key reauthentication tests now require the named provider warning.
- Duplicate-slug real-key test requires the exact provider in chat status.
- New off-origin Apps boundary journey: the account token must not be sent to a
  localhost substitute, its 403 must not poison account health, and another model
  route stays usable. This is not the trusted-service expiry/recovery case.
- Invalid-input probe requires storage to settle, rejects empty/whitespace input,
  verifies masking and requires explicit “not verified” guidance.
- Existing-profile runner records before/after configuration hashes.
- Three preview snapshots use an explicit project-name fixture instead of the
  host-dependent temporary directory; existing expected snapshots are retained.

## Evidence boundaries

The first expanded unit run generated expected snapshots. A strict repeat with
snapshot updates disabled passed all 226 selected tests. Packaged tests
must use the recorded new binary hashes, not previous candidates. Unchanged runtime
resources are copied from the prior complete package; every compiled executable is
replaced, including Code Mode and wallet helpers. Mac binaries are Developer ID
signed; no Keychain ACL or credential storage changes are made.

Passing these targeted checks does not complete all 26 multi-provider, multi-platform
journeys. Live account/billing, native Applications launch/consent and all supported
provider credentials still require their own evidence. Environment-owned key changes
require a new launching environment; blanket no-restart expectations need an explicit
scope clarification, not silent credential migration. Seven approved review passes
were exhausted before this repair; the user approved one final evidence pass.
No merge, release or human acceptance is claimed.

## Initial Apps fixture failures

The initial new Apps-only recovery test failed three times (including a diagnostic
build) before reaching its intended recovery journey. Its localhost substitution
was not a trusted ChatGPT origin: the product correctly downgraded the connection
to independent MCP OAuth and withheld the account header. The test therefore could
not establish an expired OpenAI account. No origin guard was weakened; temporary
metadata-only diagnostic logging was removed. The final off-origin regression is
a different, explicitly named expectation. F22's trusted Apps expiry/recovery
journey remains unverified, not retroactively passed by this boundary test.

The first boundary-test run also failed in a reused setup helper: it incorrectly
required an eligibility write for an account seeded before startup. The read-only
status assertion replaced that helper. Its final separate run passed on the
unchanged packaged Linux candidate (`linux/origin-test.log`). Total current
execution is the 32-check matrix plus this one additional test, not a single
33-check run. No failed attempt is counted as a success.
