## Remote work: every network refusal ends the command as failed

- **A network request the remote environment's proxy refuses on its own now
  ends the command as failed**, with the reason, as it does locally (#722,
  D278). This covers blocked sites, local or private addresses, requests
  limited mode doesn't allow, and requests to sites not on the allowlist when
  no managed network rules are set. Before, these were refused but the
  command could still end normally.
- **Only stricter:** the remote side can only report a refusal; nothing it
  sends can allow a request. Remote environments running an older version
  don't send these reports, and behave as before.
- **Hidden characters can't disguise a remote site's name** (#722): bidi
  controls and zero-width characters are removed from the site named in a
  remote refusal, and a remote request whose site name contains them is
  refused without asking.
