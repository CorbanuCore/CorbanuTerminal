# Local controller checkpoint dependency

PF20S03 supplies the staged Linux durable-root and native-channel dependency.
It does not install a privileged controller, qualify worker isolation or activate
protected mode. See [the consumer handoff](../../qa/security-levels/sprints/PF-20-S03/pf27-consumer-handoff.md)
and [verification evidence](../../qa/security-levels/sprints/PF-20-S03/verification.md).

## Required native test environment

Run native proof on the selected Linux host with `TMPDIR` on a local ext-family
or XFS filesystem. Fixtures intentionally use the production filesystem gate:
tmpfs, btrfs, overlay, network and unknown filesystems return `RootError::Unsupported`.
If `/tmp` uses one of those filesystems, the default `tempfile::tempdir()` location
cannot run these tests. Failure is not silently converted into passing/skipped
qualification. A non-Linux run likewise supplies no Linux native proof.

Use an existing approved scratch directory on a supported filesystem. For the
allocated RTX test host, the configured scratch root is
`/home/travis/security-round5/anchor-tmp`:

```sh
export TMPDIR="$(mktemp -d /home/travis/security-round5/anchor-tmp/pf20.XXXXXX)"
just test -p codex-protected-state --retries 0 --test-threads 4
```

The round's build coordinator must also hold the shared build lock and set the
approved Cargo target/toolchain environment; exact commands are in the linked QA.
Do not reformat a disk, alter system ACLs or install a controller to make a test
pass. Unsupported native APIs, permissions or filesystem semantics fail closed.

The fixtures are unprivileged synthetic data. Their real subprocess/socket and
fault-injection results do not replace two-principal deployment or physical
power-loss qualification. Two ignored child-helper tests are invoked explicitly
by their parent subprocess tests, not omitted evidence.
