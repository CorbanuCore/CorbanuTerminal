# Reproduce the revision

Run from the repository root on macOS. These are developer debug fixtures; they
are not a packaged release/native credential lane. Read
docs/development/test-isolation.md first. The build target is dedicated to this
accounting checkout and shared by all lanes.

1. Run `python3 qa/portfolio/agent-cost-accounting/pf-60-s03/acct-qualify-55/campaign.py`.
   It builds the CLI and rmcp prerequisite binaries, runs all three specified
   guarded Rust test lanes, then builds the developer-accounting CLI.
2. Compile the test-only clock shim:
   `clang -dynamiclib -o qa/portfolio/agent-cost-accounting/pf-60-s03/acct-qualify-55/fixture_clock.dylib qa/portfolio/agent-cost-accounting/pf-60-s03/acct-qualify-55/fixture_clock.c`.
3. Run `uv run --offline --script qa/portfolio/agent-cost-accounting/pf-60-s03/acct-qualify-55/qualify.py qa/portfolio/agent-cost-accounting/pf-60-s03/acct-activation-33/feature/target/debug/codex qa/portfolio/agent-cost-accounting/pf-60-s03/acct-qualify-55/NEW-RUN --mode collect`.
   Choose a new output directory; existing attempts are never overwritten.
   The pinned Python dependencies must be cached for offline use.
4. Run `python3 qa/portfolio/agent-cost-accounting/pf-60-s03/acct-qualify-55/verify_utc_alignment.py`
   to write utc-alignment-regenerated.json in acct-qualify-55 and compare its
   bytes with the untouched historical UTC-label receipt; mismatch fails.
5. Run `python3 qa/portfolio/agent-cost-accounting/pf-60-s03/acct-qualify-55/audit.py PATH-TO-COMPLETED-RUN NEW-RECEIPT.json`
   for exact Decimal equality across all rendered numeric pages.
6. Run `python3 qa/portfolio/agent-cost-accounting/pf-60-s03/acct-qualify-55/summarize.py`
   to preserve closed lane logs and refresh the inventory.

The qualification driver reuses initialization and key-navigation helpers from
the checked-in acct-qualify-50/qualify.py. It does not run that script's old test
cases. Each run preserves the expanded helpers, current driver, helper digest,
binary digest, clock-source digest, and (for later runs) clock source/binary
digest. Every expected amount is independent Decimal arithmetic from loopback
server emissions and hardcoded rates. Read-back prices are used only to check
binding against those constants. Expected membership uses the server's unique
input counts, model, and chosen exact UTC timestamp; IDs are observed through
the rendered request/attempt pages and never supply expected costs. The numeric
audit uses the driver-recorded windows in results.json for membership and checks
rendered bucket labels against them, without trusting results.passed. Arithmetic
is independent of storage; native identity and ancestry are established using
store reads, so this is not an independent proof of ancestry.

The exact clock freezes Rust wall-clock timestamps, not monotonic timers.
gettimeofday remains real because parking_lot uses it for kernel absolute wait
deadlines. The Oct 2 checkpoint is synthetic and deliberately after both tested
months. No machine clock changes. No cost row, token observation or price binding
is fabricated or edited. Only the separately disclosed ancestry scenarios mutate
native thread source/edge metadata of a genuinely sampled unrelated request.
All normal cases and metadata faults are observed through actual TUI keys.

Raw PTY streams and viewports are losslessly preserved as .gz files. Synthetic
profiles and uncompressed raw originals remain ignored. Credential files and
provider request headers/bodies are not included in the evidence.
