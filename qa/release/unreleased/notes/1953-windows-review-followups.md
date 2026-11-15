## Windows sandbox

- The elevated setup helper refuses reparse points in deny-write paths and keeps the checked paths open until their ACLs are applied. A junction swapped into a protected path cannot redirect folder creation or the deny to its target (#1953).
- The one-time temp-cache cleanup leaves ordinary reparse-point files in place. Only directories and links that redirect name resolution count as caches; the existing cache-name list is unchanged (#1953, D456).
