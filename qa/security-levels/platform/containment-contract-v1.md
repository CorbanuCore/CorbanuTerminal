# Platform containment contract v1

Version: `corbanu.platform-containment/v1`

## Authority boundary

The human controller and credential broker are trusted OS principals/processes.
Agent shells, tools, plugins, MCP servers, hooks, provider panes, child agents,
and model-controlled subprocesses are untrusted workers. Same-user process
separation, installation, notarization, code signing, a configuration flag, or a
proxy environment variable is never sufficient evidence.

Protected activation fails visibly unless every required capability is
`supported` for the current boot/target identity by a non-stale result. Unknown,
unsupported, untested, malformed, duplicated, missing, future-dated, stale, or
wrong-target input denies. A probe result cannot itself enable a broker, create a
credential route, or grant authority.

The Rust activation gate consumes the complete report envelope, an independently
derived expected target identity, and the controller's current time. A capability
slice alone is never activation evidence. Archival validation that intentionally
omits target binding is evidence QA only and cannot authorize protected mode.

## Human-controller IPC

The production IPC design must use an OS-authenticated peer identity plus a
random, single-run channel secret held outside worker-readable state. Each request
binds protocol version, controller instance, worker instance, monotonically
increasing sequence, operation, credential reference, exact destination/method/
path, expiry, and nonce under an authenticated transcript. Replays, gaps,
cross-run messages, unknown fields, downgrade, reconnect after revocation, and
identity mismatch fail before resolution. The broker returns a typed decision or
secret-free receipt, never raw secret bytes.

## Protected policy store

PF-20 must select a store owned by a principal the worker cannot read, write,
delete, rename, relink, debug, or roll back. Records bind schema version,
monotonic generation, previous-record digest, controller identity, and integrity
tag. Startup verifies the complete chain before use. Symlinks/reparse points,
non-atomic replacement, stale generation, missing head, duplicate head, restore
from an old snapshot, ownership drift, and interrupted migration all block
protected activation and require an authenticated human recovery path.

No password is persisted for elevation. Setup that needs elevation is explicit,
human approved, transactional, and independently re-audited after restart.

## Platform mechanism selection gate

PF-27-S03 records observations, not a universal mechanism choice. PF-27-S04 and
PF-27-S02 must select reviewed Linux/macOS/Windows mechanisms, rerun every probe
against the actual broker/worker launch path, and demonstrate successful denial.
An unavailable target is `untested`, not a pass. Any unsupported target keeps
Moderate/Aggressive protected activation unavailable while Permissive behavior
remains unchanged.
