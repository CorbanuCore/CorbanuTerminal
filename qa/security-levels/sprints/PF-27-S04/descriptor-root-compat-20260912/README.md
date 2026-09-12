# PF27 descriptor identity compatibility — work in progress

UNQUALIFIED. Allocation imported before code at launch
`8eeabe2cc178d41f0fec9ce65c233b062bb4a08f`; relay amendment imported as
`218124bc1`. Source baseline remains `6b393f134f83210be5a55829b63dc513f975fe57`.
Eleven allocated paths, hard800 including this receipt, fixture and runner.

New sealed child identity and additive PF20 entry remain Linux/GNU,
synthetic-feature-only, default OFF. Sole termination/reaping ownership is
unchanged. Both entries use the existing framed protocol and ambiguity handling.
No root.rs composition, system enrollment, credentials or protected activation.
Internal-only policy1.7 N/A is manager accepted; later protected-user/PF26
independent isolated-functional/native qualification remains mandatory.

Preliminary RTX evidence is preserved at
`/home/travis/security-round5/evidence/pf27-root-compat-20260912`.
Initial compilation failed on test-only WaitIdStatus equality; repaired check
and transport compilation pass. Initial full formatting lacked uv in PATH;
Rust formatting succeeded, PATH corrected, subsequent full fmt and fix pass.
Relay static hash/profile inspection and two actual-kernel identity tests pass.
First four transport cases failed before launch because /tmp is tmpfs, correctly
unsupported by PF20; they are rerun separately on a recorded ext-family TMPDIR.
Original attempts are not overwritten. Final proof/reviews33/34 remain pending.
