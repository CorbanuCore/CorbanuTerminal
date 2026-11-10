## Custom providers on OpenAI's regional API hosts

- **Multi-agent turns work on more OpenAI hosts.** A custom
  `wire_api = "responses"` provider whose `base_url` is
  `https://<region>.api.openai.com/v1` (`ae`, `au`, `ca`, `eu`, `gb`, `in`,
  `jp`, `kr`, `sg`, `us`) or `https://api.openai.com./v1` no longer gets every
  turn refused with HTTP 400 `collaboration.followup_task is reserved` (#863,
  #820). Its provider id, credentials and billing are unchanged. Proxies and
  gateways in front of OpenAI still get the provider-neutral tools.
