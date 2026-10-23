All work is in `increment-31-glm/`, both VMs are stopped (clean state), and no other files were touched. Here is my final report.

---

## PF-83 increment-31-glm — final report

### Executive summary: BLOCKED at the environmental gate. No case ran.

I could not determine whether GLM 5.2 works as the code-blind executor for F05–F11, because **the sealed guest was unreachable** and the gate never opened. Not even the F01/F02 controls could run. This is an environmental blocker, not a model refusal, not a sandbox/guard trip, and not a harness defect.

### What I did find and verify

**1. Where the harness selects the executor model (the only in-scope change).**
The executor is launched in `increment-28/driver.py`, function `guest()`:
```python
argv = [str(root/"package/corbanu"), "--no-alt-screen",
    "-c", 'model_provider="openai"', "-c", 'cli_auth_credentials_store="file"',
    "-c", 'log_dir='+..., "-C", str(run/"fixture/workspace")]
```
Provider `openai`, no `-m` → defaults to GPT-5.6-Sol (the model that refuses F05–F11). The verified change, executor only, is to add `-m glm-5.2` and set `model_provider="zai"`. I confirmed against the candidate source that a bare `glm-5.2` slug otherwise resolves to the OpenAI provider (`config_tests.rs:12466`), so the explicit `model_provider="zai"` is required to pin it. The binary supports `zai`/`ZAI_API_KEY`. Nothing else changes (same package, profile, fixture, guards, isolation, frozen cases). Full detail in `increment-31-glm/executor-model-change.md`.

**2. Gate measurements — all NOT MEASURED (guest unreachable).**

| Gate condition | Status | Reason |
|---|---|---|
| Source hosts denied from guest | not measured | denial probe runs inside the guest over SSH |
| Inference reachable | not measured | requires launching the binary in the guest |
| Native-prompt guard clear | not measured | guard runs in the guest |

Guest reachability (measured, FAILED): both UTM VMs were stopped. I started `macOS 2`; it came up at 192.168.64.3 (ICMP 0% loss) but sshd on :22 never opened after ~4 min of polling. These macOS guests need an interactive graphical login to start launchd Remote Login — an operator action I cannot do headlessly. The SSH private key at `/private/tmp/fmgr.Q1SIYZ/pf83-vm/id_ed25519` was cleaned by tmp cleanup; no copy exists anywhere I can find, and the harness enforces key-only auth with no fallback. I stopped both VMs to leave clean state. Full detail in `increment-31-glm/gate-measurements.md` and `increment-31-glm/BLOCKED.md`.

### Case table

| Case | Attempted/Refused | Harness outcome | Verdict | Evidence path |
|---|---|---|---|---|
| F01 (control) | not reached | not dispatched | not_reached | (no run; gate closed) |
| F02 (control) | not reached | not dispatched | not_reached | (no run; gate closed) |
| F05 | not reached | not dispatched | not_reached | (no run; gate closed) |
| F06 | not reached | not dispatched | not_reached | (no run; gate closed) |
| F07 | not reached | not dispatched | not_reached | (no run; gate closed) |
| F08 | not reached | not dispatched | not_reached | (no run; gate closed) |
| F09 | not reached | not dispatched | not_reached | (no run; gate closed) |
| F10 | not reached | not dispatched | not_reached | (no run; gate closed) |
| F11 | not reached | not dispatched | not_reached | (no run; gate closed) |

No GLM 5.2 refusal was observed because GLM 5.2 was never launched. There is no verbatim refusal to record because no inference call occurred.

### Exact executor model configuration used

**None was executed.** The intended configuration (verified, staged, not run) is `-m glm-5.2` with `-c 'model_provider="zai"'`, applied only in the `guest()` launcher. The run used no executor model because it never started.

### "Can GLM 5.2 serve as the code-blind executor for F05–F11?"

**Undetermined — not a yes or no.** The experiment did not run, so there is no evidence either way. I will not infer a verdict from the absence of a run. The question can only be answered once the operator restores three things, none of which the harness can self-provision:

1. **Start the sealed guest VM** and complete the graphical login so sshd comes up at `agent@192.168.64.3:22` (or confirm which VM is the PF-83 guest and that Remote Login is enabled).
2. **Restore the guest SSH private key** to a stable (non-`/tmp`) path and repoint the staged environments' `identity_file`.
3. **Provision a zai inference credential** through the harness's mediated owner path. The existing `/Users/agent/.pf83-auth/auth.json` was staged for OpenAI; whether it already carries a zai key is unknown and unverifiable from the host without reading the credential, which the hard rules forbid. If it does not, a zai credential must be added before any GLM 5.2 run.

### Integrity

- No case text, expected result, or validator was edited.
- No guard, sandbox rule, or isolation setting was weakened; no guard tripped.
- No credential was read, copied, printed, or accessed; the operator's Keychain was untouched.
- No repository source was edited and nothing was pushed.
- All output is in the new write-once directory `increment-31-glm/` (README.md, BLOCKED.md, executor-model-change.md, gate-measurements.md). No prior capture was overwritten.