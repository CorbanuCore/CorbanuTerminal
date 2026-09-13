# Private dashboard relay — qualified backend, not HTTPS

PF-80-S01 internal operator configuration under **Internal delivery control —
TO BUILD**, “private read-only Alex-server dashboard”. These are the exact reviewed
and tested user units, not an automatic installer or enabled service.

Direct RPC SSH works without Tailscale. Existing dashboard127.0.0.1:8768 requires
a localhost Host header. Tailscale1.98.10 preserves incoming Host for TCP backends
but uses localhost for Unix backends. A private socket relay preserves the
dashboard's existing Host/path allowlists; no public listener or nginx change.

Installed in `/home/pfrpc/.config/systemd/user/`; socket path
`/home/pfrpc/corbanu-control/private-serve/dashboard.sock`, directory0700/socket0600.
Systemd255 unit verification passed. Fable review01 found TasksMax8 insufficient
for up to16 resolver workers plus main; corrected to32. Corrective review02 clean.
Exact unit SHA256: socket `c42402c51864ee397a0409bd7c13b61e6a3de3894ce6dba5e5d63542eb3c1558`;
service `8d5613cbc5ee1178e875479b24d9594ae50343eba9a4efc2ad23cc3aec59f13d`.

Actual direct-SSH qualification by fresh Astra High Pascal:

- Facilities200 with X-Corbanu-Control1; untrusted Host403.
- Literal/encoded traversal404 matched direct backend rejection/body hashes.
- 16/16 barrier-released concurrent requests200 with identical content/marker.
- Existing web PID/invocation unchanged; publisher timer disabled/inactive;
  Tailscaled unchanged; Serve/Funnel configurations remained empty.
- Exact relay units stopped; socket removed. Installed socket disabled, service
  static; neither enabled. Private directory remains0700. Worker closed.

Original failures retained: first local rsync option rejected before transfer;
corrected syntax transferred only two units. Initial traversal assertion expected
403; actual404 was independently compared with backend404 and accepted as rejection,
without rerunning the relay or loosening allowlists. Raw command/exit/status/hash
receipts: `.codex-work/private-relay-qualification.X4WO1u/`. Parent inspected the
raw HTTP records, shutdown verification and reviewed input hashes.

Remaining: publisher-tailnet administrator sign-in and Serve enablement; approved
private HTTPS route and off-Mac/unauthorized-access/recovery tests. Do not enable
Funnel, broaden ACLs or expose the loopback dashboard. The browser setup page
still showed administrator sign-in at the last inspection. Local relay success
does not establish HTTPS, phone access, Slack acceptance or whole-sprint completion.
