# Harness (coordinator-side; not product code)

| File | Version used | SHA-256 |
| --- | --- | --- |
| `agms-product`, `agms-tui` | v1: original attempts G01-G20 (installed by provision run 5) | `d0830359…4a7b`, `b9423eaf…3984` |
| `v2/agms-product`, `v2/agms-tui` | v2: replays and G21 (provision run 6); `--env` accepts any letters/digits/`.`/`_`/`-` value, help text says so | `8c2803ff…a74`, `9acbd38f…d17a` |

Installed digests per provisioning run are in `../common/provision-*.out`. A coordinator control that a `--env`
value reaches the product process under v2 is in `../common/fixture-v2-control.txt`.
`restore.sh` reverses `provision.sh`; `admin.sh`/`su.exp` run one root script with the admin password piped from
`corbanu vault auth-helper neo-vm` and never echo it.
