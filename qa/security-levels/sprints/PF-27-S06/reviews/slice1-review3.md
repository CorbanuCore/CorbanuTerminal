VERDICT: APPROVE

I reviewed only the diff (`6a0a5f8d62..HEAD`). I did not build or run anything. I took CI job 113152390863 as reported.

## Second-review findings

| # | Status |
|---|---|
| A, B | **Accepted as documented limits.** The comment at `windows_process_access.rs:16-23` describes the threat scope accurately. The remaining item is in Low 1. |
| C | **Fixed.** See the PE walk check below. |
| D | **Fixed.** The callback counts failures at `:216-218`, and the hardened target asserts the count is zero. |
| F, H | **Fixed.** The control asserts the report length, and a granted right now names the thread (`granted@{tid}`). |
| G | **Not done (optional).** Nothing measures the race. Acceptable now that the gap is documented. |

## PE walk check (`windows_process_access.rs:254-293`)

- **Header offsets are correct.**
  - The signature check at `e_lfanew`, then `optional = nt + 24`.
  - PE32+ (`0x20b`): the directory count is at +108 and the directories start at +112.
  - PE32 (`0x10b`): the count is at +92 and the directories start at +96.
  - The TLS directory is index 9, and its entries are 8 bytes each.
- **The `AddressOfCallBacks` offset is correct.** It is at +24 in PE32+ (three u64 fields come first) and at +12 in PE32 (three u32 fields come first).
- **Treating it as a VA is correct.** The field is a VA that the loader has already relocated, so it is right to dereference it directly rather than add it to `base`. The callback array entries are also relocated VAs, so they compare correctly with the pointer value in `THREAD_ATTACH_CALLBACK`.
- **The pointer width is correct for both formats.** Reading entries as `usize` matches the image's pointer size: an x64 Rust binary is PE32+, and an x86 one is PE32.
- **The walk always terminates.** It stops at a null entry or after 64 entries, and returns empty on a bad magic, too few directories or no TLS directory. All of these fail closed, so hardening refuses.
- **The volatile read keeps the callback linked.** It references the `.CRT$XLC` static from `restrict_current_process_access`, so `/OPT:REF` keeps the section whenever Core calls the hardening.

## Low

1. **The A/B limit has no recorded owner sign-off.** The module comment documents the gap, but there is no sign-off from the security owner. My second review gave that sign-off as the minimum condition.
   - **Fix:** record the acceptance, with the scope "same-user unsandboxed processes are not contained (PF-27-S02)", in the PR or sprint evidence before the sprint is closed.

2. **No bounds checks against the image size** (`:256-283`). `e_lfanew` and `tls_rva` are not checked against `SizeOfHeaders` or `SizeOfImage`. The input is the process's own loader-mapped image, so this is not exploitable; it is defence in depth.
   - **Fix:** read `SizeOfImage` (optional header +56) and return empty if `nt + 24 + 112 + 80` or `tls_rva + 32` is larger than it.

3. **The registration is only proven for the test binary.** CI shows the callback in the test binary's TLS directory. In the release Core binary, the runtime check refuses a launch if the callback is missing, which is safe, but a linker regression would only show up when users hit it.
   - **Fix:** add a release-smoke step that runs the packaged Core on Windows and asserts that `thread_callback_registered()` is true and `thread_protection_failures() == 0` after a thread starts.

4. **Minor robustness.** `read_u32` builds unaligned reads from `base` with no provenance check. That is fine on Windows. A short note that the image is the one containing this code (so it cannot be unmapped while we read it) would help future readers.

There are no correctness or security blockers in the new code.
