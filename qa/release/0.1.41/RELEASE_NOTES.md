Corbanu Terminal 0.1.41 fixes Task Node account recovery when using multiple named profiles on one machine.

- Each profile keeps its own independently linked session.
- Completing `/tasknode link` replaces a stale saved session when you run `/tasknode status`.
- Status shows the current profile, and CLI linking instructions keep the selected profile.
- Authentication errors give current Corbanu instructions.

The live Task Node linking service also now opens GitHub's account picker. If a saved session was revoked, restart Corbanu, run `/tasknode link`, choose the intended GitHub account and then run `/tasknode status`.

The repair passed 48 focused Rust tests and actual terminal checks covering two-account isolation, relinking, restart, cancellation and concurrent status. This emergency release is authorized with the full competitor/model benchmark matrix incomplete. [Release evidence](https://github.com/CorbanuCore/CorbanuTerminal/blob/rust-v0.1.41/qa/release/0.1.41/RELEASE.md).
