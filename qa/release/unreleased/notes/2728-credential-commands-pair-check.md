## Fixes

- `corbanu login`, `corbanu login status` and `corbanu logout` (also with `--account`) now work when the configured model does not belong to the selected provider, for example `amazon-bedrock` with `gpt-5.6-sol`. Before, they stopped with "explicit model/provider pair ... is incompatible", so you could not sign out until you fixed the model. Other commands still refuse that configuration. Your configuration file is not changed.
