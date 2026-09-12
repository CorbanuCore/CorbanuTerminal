# Native attempt: blocked, launch outcome unconfirmed

Luna Max successfully called Computer Use, observed Finder Applications, and
captured native screenshots in its tool transcript. It attempted to open the
Applications launcher. It initially reported a launch failure because no new
window was observed in Finder and the launcher did not remain running.

That initial attribution is **not sufficient**: an applet may exit after opening
a Terminal window on another desktop. The coordinator requested only a native
Terminal window-inventory observation, with no additional launches or shell
interaction. Computer Use refused Terminal access for safety. The agent stopped
and amended its report to **unconfirmed**, not a definite zero-window failure.

Consequently, launch, Desktop 3 placement, and three-window concurrency remain
blocked in this native attempt. The user's earlier report of the second window
landing on the wrong desktop remains open; this attempt does not supersede it.

A separate read-only filesystem check confirmed that the 69-byte Applications
item is an expected symlink to an existing app bundle and executable. The alias
label alone is not evidence of a broken shortcut. The stable binary link still
resolves to the signed candidate with SHA-256
`4a8eba7b10199ea49aee42a720687b63cb4b1b2194c2ce164d1b9f6f88ee510e`;
strict signature verification succeeded. That verifies the file, not what any
native launch executed.

See [the original action record and appended correction](native-execution.md).
No permission changes or alternate route around the Terminal restriction was
attempted. Desktop code blindness was instruction-only. No product fix or new
independent review is claimed.
