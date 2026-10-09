# PF-27-S09 evidence: Windows model-client auth through the broker

The port of [PF-27-S05](../PF-27-S05/README.md) to Windows. It is behind the default-off `broker_model_auth` flag;
Permissive, macOS and Linux are unchanged. Stored keys follow Travis's key-path decision (c) of 2026-10-09: the
broker reads the vault key from Credential Manager itself, under its own PF-27-S08 token.

## What shipped

- **Slice 1 (#363), the broker side** (`network-proxy`, `vault`, `arg0`):
  - **Env keys.** When Core hands a provider key to the broker, it overwrites the key in place in the process
    environment block (same-length `000…`). It also drops the key from the C runtime's tables and sweeps every
    process heap, giving each copy of the value `0` characters:
    - `NAME=value` entries and copies in free blocks;
    - bare copies in blocks in use, for values of 20 characters or more.

    It also overwrites `NAME=value` entries in the allocation that holds the process parameters. When the
    environment grows, Windows moves the environment block but cannot free the original launch block, which
    shares that allocation.
  - **Broker launch.** The broker's launch environment is built without copying the values Core withholds.
  - **Stored keys** are read inside the broker by the binary's resolver:
    - Core creates `secrets/.vault.lock` before the broker starts. It refuses reparse points and checks the path
      of the handle it opened.
    - The broker's token cannot open that lock for writing, so the vault locks it through a read-only handle. This
      fallback exists on Windows only.
  - **Requests.** `ModelCredentialBroker::send` sends a signed request over the broker's data pipe. Every
    connection is checked to be served by the broker process (PF-27-S06).
  - **Platform roots** now load read-only. Under the broker token, `schannel` could not open the root store for
    writing, so the broker trusted no root. Brokered HTTPS then failed with `UnknownIssuer`, which only the real
    machine showed.
- **Slice 2, Core** (`core`, `http-client`):
  - The Windows start spawns the broker, hands over the env keys and installs a `ModelBrokerSender` (`PipeSender`)
    as the route for every frame-bearing request.
  - reqwest's named-pipe connector cannot check the server process, so the sender uses slice 1's checked pipe.
  - With no route installed, frame-bearing requests are still refused.

## Measured

On a real Windows 11 machine, 2026-10-09, using `C:\CorbanuQA\s09`. Slice 1 was tested at `6e767d417`; the final
product runs and videos used `cf3d8b352a` (slice 2, with slice 1 merged in):

| What | Elevated (SSH) | Normal session (console, medium) |
| --- | --- | --- |
| network-proxy lib tests (including the `pf_27_s05` suite over pipes and 12 `pf_27_s09`), vault, arg0 | 321, 59, 9 passed | 321, 59 passed |
| `credential_broker` tests in parallel, as the PF-13 credential canary runs them | 5 of 5 runs passed (71 tests) | — |
| http-client and Core `model_broker_auth` | 1 + 6 passed | 1 + 6 passed |
| Memory scan, after Core's environment has grown: hits before → after hand-over | found (ASCII and UTF-16) → 0 | same |
| Read-only lock probe under the broker token | write-open `PermissionDenied`; read-only lock works | same |
| Platform roots under the broker token | the same count as an unconfined process | same |
| `corbanu exec`, GLM 5.2, env key | answers; key handed over and removed; `token+dacl+job` | same |
| `corbanu exec`, GLM 5.2, vault key | answers, with the vault key in the profile's file fallback (see limits) | answers, with the vault key in Credential Manager and no fallback file |
| Elevated sandbox command (`whoami`) with brokered auth | `codexsandboxoffline` | not run (one-time admin setup needed) |
| Heap sweep per handed-over key (debug build) | 138 ms | 148 ms |

- **Clippy `-D warnings`:** clean on Windows for network-proxy, vault, arg0, http-client and core, and on Linux on
  the RTX box for the same crates. Linux tests pass: network-proxy 298, vault 59, arg0 10, http-client 73, Core 5.
- **Reviews (Opus 5.5 High):**
  - slice 1: APPROVE, then three scoped re-reviews, all APPROVE: the fixes and the roots change; the busy-block
    sweep; the parameters' allocation;
  - slice 2: APPROVE.
  - The non-blocking findings were fixed, or are listed below.
- **CI found two leftover copies, both fixed in #363.**
  - **The PF-13 credential canary** on `windows-2022` once found a leftover in the memory-scan child; that commit
    passed elsewhere. The C runtime frees its start-up copies of the environment unwiped, and reused memory can
    keep the value without its `NAME=` prefix, in a block that is in use. The sweep now covers bare copies in
    blocks in use.
  - **The normal-session run** of `windows-security-probes` found a UTF-16 `NAME=value` entry outside every heap:
    the launch environment block, which Windows replaced when the environment grew. The sweep now covers the
    parameters' allocation.
  - Deterministic `pf_27_s09` tests pin both cases.
- **SOP videos:** GLM 5.2, leak-scanned (no key value, no key-shaped string), listed in
  [qa/demos/index/PF-27-S09.md](../../../demos/index/PF-27-S09.md):
  - env key in a normal session;
  - vault key in a normal session;
  - over SSH with the elevated sandbox.

## Known limits

- **No Core-level memory scan.**
  - The scan covers the hand-over path in a fresh process whose environment has grown, as PF-27-S05's did. It does
    not cover Core's own start-up.
  - A bare copy of the key that other Core code keeps in a live block (for example a `std::env::vars()` snapshot)
    is not found. The sweep only rewrites `NAME=value` entries and free blocks.
- **The heap sweep is best effort.**
  - It writes `0` characters into other code's live blocks wherever a whole value of 20 or more characters
    matches. Any other copy of the key that Core held is lost on purpose, because Core must not use the key after
    the hand-over.
  - Between the check and the write, the owner of a live block could reuse those bytes, and the sweep would then
    overwrite new data. The window is tiny, and it is not closed.
  - `HeapLock` does not lock a heap created with `HEAP_NO_SERIALIZE`, so walking one races with its owner.
  - It costs about 140 ms per key at session start (debug build).
  - Outside the heaps, only the parameters' allocation is swept: for example, a copy on a thread stack is not
    found.
- **Over SSH there is no Credential Manager.** In an OpenSSH session authenticated with a key, the vault keeps its
  key in the profile's file fallback. This was already the case before this sprint. The broker reads that file the
  same way. In the console session the key is in Credential Manager and the broker reads it.
- **Decision (c)** (PF-27-S08): the broker's token can read and write the user's other generic credentials.
- **ChatGPT sign-in.** Replacing the broker's copy on a token refresh is the same code as on Unix and is covered by
  unit tests only. The machine has no ChatGPT login.
- **Request headers.** The Windows sender adds Core's default headers and `Accept`. It does not add reqwest's
  tracing headers or the request debug log.
- **Vault lock.** If `secrets` is swapped for a junction at just the wrong moment, Core can create an empty lock
  file at the junction's target. It then rejects that file.
- **Inherited limits.** PF-27-S05's known limits still apply, for example other features that open the vault
  in-process.
