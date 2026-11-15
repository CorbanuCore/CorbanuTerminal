## Windows sandbox

- The elevated setup helper refuses redirecting links in deny-write paths. It creates missing folders relative to checked handles and applies ACLs to the opened object, so a junction swap cannot redirect creation or the deny to its target. Non-redirecting reparse points, such as OneDrive folders, remain supported (#1953).
- The one-time temp-cache cleanup leaves ordinary reparse-point files in place. Only directories and links that redirect name resolution count as caches; the existing cache-name list is unchanged (#1953, D456).
