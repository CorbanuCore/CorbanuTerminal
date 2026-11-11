## OpenRouter Tencent Hy3 Free is retired

- **`tencent/hy3:free` is no longer listed in `/model`** (#1658, #1662, thanks
  to @bradsbots). OpenRouter lists no endpoints for it (checked 2026-11-07), so
  a request to it is expected to fail at OpenRouter, and it is no longer
  allocated automatically. A config that saved this ID still resolves to its
  own hidden entry and is never switched to another model; its description now
  says it is retired. Pick a current model with `/model`. The paid
  `tencent/hy3` is not in the catalogue.
