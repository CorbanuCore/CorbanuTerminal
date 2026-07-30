# Upstream Convergence Release Baseline

Captured: 2026-07-30 UTC

This file freezes the released state that the upstream convergence work must
remain able to open and supersede.

## Source

- Release merge: `d9e2a383ab02550ba26c525c6c99794dd99ae13a`
- Release branch tip: `5e79527205440cd0447200c1fcb191ad8140d0ea`
- Version: `0.1.26`
- Integration target: OpenAI Codex
  `413492cd6c3a4d4f8dff6f406247ccda5a9d88aa`
- Common ancestor:
  `d66708232299bdbf373ec55b0d6b938c246cfa60`

## Installed rollback artifact

- Path:
  `/home/pfrpc/.pfterminal/packages/standalone/releases/0.1.26-x86_64-unknown-linux-gnu`
- Binary SHA-256:
  `0d3647dcc7cfdffab95f094f21fc50a59954b4d964fc2e65500d64f02e18160c`
- The standalone `current` symlink resolved to that directory when this
  baseline was captured.

The convergence work must not modify this installed release directory or move
the stable `current` pointer during debug qualification.

## Source artifact hashes

- `codex-rs/core/config.schema.json`:
  `a0f4db95ce74049bcc4c745e90f0491d4f2166f61c49355a711f42ecbe7e4dcb`
- `codex-rs/Cargo.lock`:
  `1ceafe1f95e2a44cd282c746d21ca74403b64826882026acc9cf017e669f4cc1`
- `README.md`:
  `b64aab34d00895d907bfaab9a54613b85737f6698aefd24d1b5e01c10d4f2ac4`

## Database compatibility boundary

Released PF migrations extend through `0045_agent_mailbox.sql`. Every migration
through version 45 must keep the exact Git blob recorded by release merge
`d9e2a383a`. The convergence branch may add later migrations but must never
rewrite those files.

The authoritative baseline is:

```sh
git ls-tree -r d9e2a383a -- codex-rs/state/migrations
```

Required fixtures:

- a copy of a 0.1.24 home;
- a copy of a 0.1.25 home;
- a copy of the active 0.1.26 home;
- the foreign-version-45 collision case that previously prevented resume.

All migration and resume tests must operate on copies. Production
`~/.pfterminal` is outside the destructive test surface.

## Debug isolation

The debug wrapper executes:

```text
/home/pfrpc/repos/PfTerminal-telegram-hardening/codex-rs/target/debug/pfterminal
```

The pre-convergence debug binary was version 0.1.25 and is not release
evidence. It may be replaced by the convergence debug build after tests pass.
The installed 0.1.26 standalone binary remains the rollback artifact.
