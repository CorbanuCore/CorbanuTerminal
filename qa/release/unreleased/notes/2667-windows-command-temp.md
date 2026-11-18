## Windows sandbox

- Elevated commands choose their private session temp folder from their resolved command environment. An omitted `TMP` no longer overrides an explicit command `TEMP` with the server's folder; omitted, empty or relative temp settings do not restore server temp access. The private session folder and parent-folder protections remain in place.
