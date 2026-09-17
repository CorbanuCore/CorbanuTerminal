# Package exclusion

The three binaries are deliberately excluded local debug build products.
The repository rule is [`acct-fitness-76/.gitignore`](../acct-fitness-76/.gitignore),
whose first line is `/package/`; the
[engineering record](../acct-fitness-76/engineering.md) identifies the staged
copies as local debug artifacts rather than a signed distribution. This is
build-output exclusion policy, with a substantial storage cost, not evidence of
a licence prohibition. Git ignore rules can technically be overridden; neither
the brief nor the retained record establishes that all three files *cannot* be
committed.

| Binary | Manifest bytes | MiB |
| --- | ---: | ---: |
| codex | 607,785,336 | 579.629 |
| codex-code-mode-host | 93,937,576 | 89.586 |
| rmcp_test_server | 11,508,704 | 10.976 |
| Total | 713,231,616 | 680.191 |

These figures come from the retained
[package manifest](../acct-fitness-76/package-manifest.json); they describe
uncompressed staged files. The clean integration checkout has none of those
bytes. Thus their sizes, SHA-256 digests and mode `0555` cannot be re-derived
from that checkout. The manifest and build log are committed and checked.
The build recipe is retained in
[prepare_package.py](../acct-fitness-76/prepare_package.py), but a future rebuild
is not proof of byte-identical reproduction of the historical package.
No binaries were launched or added by this revision.
