# D319 zip compatibility fixtures — never regenerate

Generated once on 2026-11-07, before the upstream sync (`rust-v0.167.0`) touched `zip`, while `main`
(`18fa372aaaf5`) used `zip` 9.0.5 with the reduced decoder set from #968 (`deflate`, `deflate64`, `bzip2-rs`).
They stand in for the untrusted skill and plugin bundles we extract. `tests/d319_zip_compat.rs` checks them.

- `stored.zip`, `deflate.zip`, `bzip2.zip`, `rejected-lzma.zip` (method 14), `rejected-zstd.zip` (method 93):
  Python 3.14 `zipfile`.
- `deflate64.zip`: 7-Zip (`7z a -tzip -mm=Deflate64`).
- `expected/`: the files every accepted archive must extract to.

The accepted methods may grow only with the security owner's approval (then only the test's `REJECTED` list
changes) and must not shrink. SHA-256 at generation:

```
465f592ddd0bccafddbde94a6bc0ecc9c5312c422960c79d37552f18851a2819  bzip2.zip
3e2e08b216f67fd867e24de9cf314f8c29c108f8b9ab5d443ba331043a46f022  deflate.zip
e2a7d61c578f877b856e9477a2df97ea66a929fcdc982558cb17ef989236e56f  deflate64.zip
f9154706507bd0cc2baea116faeea2bacc2ed43198ff7f78ef6a0f08e3aa29df  rejected-lzma.zip
55b5c925578dea161887cf3cf872a3fa047a1a8c203ac7b345cbffe43a75bc62  rejected-zstd.zip
55079074153a5d06708ba986cc80336abd29093b5d58d931f5413fdbd6f96ed7  stored.zip
54590afc553e24dbfc5f92c4f781c051bb06345e4e687b136fe24cfa1f46092b  expected/bundle-root/README.md
986d40bcad57621d2420791c824b9d8292e7f1699ae565ee2f52a0eb32ab04bc  expected/bundle-root/skills/demo/SKILL.md
785b0751fc2c53dc14a4ce3d800e69ef9ce1009eb327ccf458afe09c242c26c9  expected/bundle-root/data/bytes.bin
```
