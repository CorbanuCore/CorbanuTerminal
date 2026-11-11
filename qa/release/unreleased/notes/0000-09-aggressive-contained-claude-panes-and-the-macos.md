## Aggressive: contained Claude panes and the macOS keychain

- **Contained Claude panes follow the stricter level right away** (#807). A
  contained pane is now kept away from the macOS keychain services as soon as
  Aggressive is saved in `/security` or takes effect in the live session,
  before any restart, as agent commands already were. On Linux, a pane's
  user-namespace rule (#891) follows the live level the same way.
- **Approved commands stay in the sandbox under the keychain rule.** On macOS
  the keychain rule by itself now keeps escalated and allow-listed commands in
  the sandbox. Aggressive already kept them there through its other rules, so
  nothing you see changes. Permissive and Moderate are unchanged.
- **Claude panes and `/permissions` also follow Core's live level** (#1468).
  Claude panes (outside containment) and permission changes are refused while
  the live session runs a protected level (Aggressive, for `/permissions`),
  not only when it is active or saved. On macOS, `/status` no longer says
  approved commands can leave the sandbox when a permission profile keeps the
  keychain away (they can't).
