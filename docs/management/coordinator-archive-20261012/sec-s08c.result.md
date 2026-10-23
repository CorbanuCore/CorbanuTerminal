I merged **PR #360** into main at `6e7564c5c6`, with all 24 checks passing and none pending; the merge was pinned to head `484424d6e2`. PF-27-S08 is now **`ready`** for Travis's acceptance. I did not archive it.

**Recovery:** the previous run had already opened #360 with the decision write-up and was waiting on checks when it died. I didn't redo that work. I updated the evidence to the newest main commit, wrapped one long line, removed a stray blank line, merged main into the branch (no force-push) and pushed one commit. The sprint and plan checkers both pass.

**What changed:**
- **S08 sprint record:**
  - Records the key-path decision: option (c), decided by Travis on 2026-10-09. The Windows broker reads Credential Manager itself, as the macOS and Linux brokers read the OS keyring (PF-27-S05).
  - Acceptance criterion 5 now says stored keys come from Credential Manager, read under the broker's own token. "The broker's token reading Credential Manager is denied" is dropped.
  - Notes that #333 was merged with five checks still pending, which later passed on main at `d0544c1c91` (windows-security-probes and corbanu-terminal-ci).
  - Status is `ready`; every item is ticked except Travis's acceptance and the archive.
- **S08 evidence README:** the open-decision section is now a decision section, there's a section on the merge and current main, and the Credential Manager limit is added under Known limits.
- **Gate evidence still holds on the latest main (`5d283fde18`):** CI run 37973587031 passes the `pf_27_s08` probes 7/7 elevated and 7/7 in a normal session, and the broker suite over the pipes 25/25 in both.
- **S09 sprint record:** it now depends on S08 and states that stored keys use decision (c), so S05 ports as is. One Windows addition: each vault read opens `secrets/.vault.lock` for writing, and the S08 token can't do that. S09 must either grant the broker access to that file or hand it an already-opened lock.

**Correction to the brief:** it says the broker's ability to read, write or delete other credentials was measured. Only read and create were measured. Overwriting or deleting an existing credential was never probed (the delete test only hit one that didn't exist). The record says to assume it can, since Credential Manager has no per-credential access control.

**Known limits Travis is accepting with S08:**
1. **Credential Manager:** a compromised broker can read the user's other generic credentials and create new ones (measured). It can probably overwrite or delete them too, including the vault key (not probed). Its other confinement limits the damage, but its network access isn't restricted, so it could send what it reads anywhere.
2. **Terminate:** it can kill the user's ordinary processes, such as the terminal or an editor. Core is protected from this.
3. **Delete in low-integrity folders** (`LocalLow`, `Temp\Low`, likely `Packages\*` and `INetCache\Low`): it can delete files there, though not create, change or rename them.
4. **Not probed:** squatting on low-labeled named objects, pipes or mailslots that grant Everyone or the logon SID; changing permissions or owner on low-labeled files; `HKCU\Software\AppDataLow`. Core also trusts the broker's own report of its token rather than checking the token itself.
5. **WMI/COM coverage is partial:** other out-of-process COM servers and the wider Windows RPC surface carry the same risk. A COM server whose launch rights grant Everyone, Users or Authenticated Users would get around the INTERACTIVE deny.
6. **Task Scheduler** refuses by hiding its folders rather than denying access. In CI's normal session the unconfined comparison process couldn't register a task either, so there that test has no working control; the elevated run has one.
7. **COM control on the real machine:** `MMC20.Application` needs elevation in a normal session, so the COM test's working comparison there is the elevated run only.
8. **Parent-process spoofing signal,** as described in the evidence README.