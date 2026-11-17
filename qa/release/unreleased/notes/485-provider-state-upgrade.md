## Provider request state

- Provider request throttling uses a per-home salt for default API-key
  fingerprints. On upgrade, active legacy cooldowns and leases are carried
  over when that credential is next used; expired idle legacy rows can be
  removed. Named-account throttling follows the credential rather than only
  the account's name (#485, #569).
