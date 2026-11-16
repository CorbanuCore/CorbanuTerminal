## Security

Command rules now also check commands run through `sudo`, `chroot`, and `env`, including nested wrappers. The strictest outer or inner decision applies, and unrecognized wrapper options require approval. This fixes wrapped commands bypassing prompt or forbidden rules (#2165, D553).
