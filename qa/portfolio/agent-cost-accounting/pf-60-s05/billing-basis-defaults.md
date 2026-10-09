# PF-60-S05: default billing basis for every built-in provider

*2026-10-08. The provider list is from `built_in_model_providers` (`codex-rs/model-provider-info/src/lib.rs`) on main
`7a11f9068b`: 21 providers. Sources were read on 2026-10-08.*

Each default is the basis that matches the configuration a user is expected to have for that provider: the built-in
URL and the credential the built-in entry asks for. "Also bills" lists cases where the same vendor can charge the other
way. Those cases fall under the open "both" decision ([options memo](both-behaviour-options.md)), not under these
defaults.

| Provider id | Built-in route and credential | Default basis | Also bills | Source |
| --- | --- | --- | --- | --- |
| `openai` | ChatGPT sign-in | subscription | Purchased credits after plan limits | [Codex pricing](https://developers.openai.com/codex/pricing): Codex is included in ChatGPT plans; extra usage is bought as credits |
| `openai` | `OPENAI_API_KEY` | pay per use | — | Same page: API-key use is paid "based on API pricing" |
| `anthropic` | `api.anthropic.com`, API key | pay per use | — | [Claude API pricing](https://platform.claude.com/docs/en/about-claude/pricing); [Claude support](https://support.claude.com/en/articles/11145838-use-claude-code-with-your-pro-or-max-plan): an API key means API usage charges, not the subscription |
| `claude-plan` | Claude Pro/Max login (command auth) | subscription | Usage credits at API rates once plan limits are reached | [Use Claude Code with your Pro or Max plan](https://support.claude.com/en/articles/11145838-use-claude-code-with-your-pro-or-max-plan); [usage credits](https://support.claude.com/en/articles/12429409-manage-usage-credits-for-paid-claude-plans) |
| `ambient` | `api.ambient.xyz/v1`, API key | pay per use | Subscriptions exist but cover web chat and Ambient Desktop | [Ambient pricing](https://ambient.xyz/pricing): subscriptions are a compute allowance for web chat and Desktop; API pricing is pay per token |
| `pfterminal-plan` (aliases `corbanu-plan`, `corbanu-terminal-plan`) | `api.corbanu.com/v1`, Corbanu API key | pay per use | — | [Product spec, "Corbanu API"](../../../../docs/corbanu-product-spec.md): a dollar balance is debited per use; legacy plan allowances were retired and deleted on 2026-08-30 |
| `pfterminal-plan-anthropic` (alias `corbanu-plan-anthropic`) | Same gateway, Messages wire | pay per use | — | Same |
| `kimi-code` | `api.kimi.com/coding/v1`, `KIMI_API_KEY` | **subscription** (today wrongly pay per use: the review's Blocker) | Extra Usage balance once quota is spent | [Kimi Code membership](https://www.kimi.com/code/docs/en/kimi-code/membership.html); [Kimi API troubleshooting](https://platform.kimi.ai/docs/guide/troubleshooting): Open Platform, Kimi Code and Membership are separate products whose balances and keys don't transfer |
| `zai` | `api.z.ai/api/paas/v4` (general endpoint), `ZAI_API_KEY` | pay per use | The GLM Coding Plan uses a different URL (`/api/coding/paas/v4`) | [Z.AI: other tools](https://docs.z.ai/scenario-example/develop-tools/others): the Coding Plan must use the coding endpoint "instead of the General API" |
| `zai-anthropic` | `api.z.ai/api/anthropic/v1`, `ZAI_API_KEY` | subscription | Balance only for accounts that have never bought a Coding Plan and are allowlisted | [ZCode: connect models](https://zcode.z.ai/en/docs/configuration): the Anthropic URL is the Coding Plan route and doesn't bill a prepaid balance once a plan has been bought. **Needs a real-provider check (AC10).** |
| `openrouter` | `openrouter.ai/api/v1`, API key | pay per use | BYOK fee above an allowance | [OpenRouter FAQ](https://openrouter.ai/docs/faq): request cost is deducted from credits |
| `openrouter-anthropic` | Same, Messages wire | pay per use | Same | Same |
| `deepseek` | `api.deepseek.com`, API key | pay per use | — | [DeepSeek pricing](https://api-docs.deepseek.com/quick_start/pricing): fees are deducted from the topped-up or granted balance |
| `meta` | `api.meta.ai/v1`, `MODEL_API_KEY` | pay per use | Consumer Muse subscriptions are separate | [Build with Muse Spark](https://developer.meta.com/ai/resources/blog/build-with-muse-spark/): Meta Model API pricing is pay-as-you-go |
| `baseten` | `inference.baseten.co/v1`, API key | pay per use | — | [Baseten pricing](https://www.baseten.co/pricing/): Model APIs are priced per 1M tokens; Basic is "$0 per month, pay as you go" |
| `baseten-anthropic` | Same, Messages wire | pay per use | — | Same |
| `vercel` | `ai-gateway.vercel.sh/v1`, API key | pay per use | $5/month free credit; BYOK | [AI Gateway pricing](https://vercel.com/docs/ai-gateway/pricing): pay-as-you-go from purchased credits |
| `vercel-anthropic` | Same, Messages wire | pay per use | Same | Same |
| `vercel-anthropic-fast` | Same, fast route | pay per use | Same | Same |
| `amazon-bedrock` | AWS-signed or bearer-token route | pay per use (also when a command `auth` is set: review Minor 10) | Provisioned throughput, batch | [Bedrock pricing](https://aws.amazon.com/bedrock/pricing/): on-demand pricing, billed to the AWS account |
| `ollama` | `localhost:11434` | local | Ollama Cloud plans exist but are not this route | [Ollama pricing](https://ollama.com/pricing): models on your own hardware are "always unlimited" |
| `lmstudio` | `localhost` LM Studio port | local | — | [LM Studio](https://lmstudio.ai/): runs models locally |

## Notes

- Free starter credit (Meta, Baseten, Vercel's free tier) doesn't change the basis. It is still money on a pay-per-use
  balance, so it is counted as spent. Reconciling free credit is out of scope.
- Third-party guides describe the Z.AI and Kimi splits the same way as the sources above. Only official vendor pages are
  cited here.
- Option B (decided 2026-10-09): the basis follows the route and credential actually used. A provider reached at a
  route or with a credential this table doesn't list is "billing basis not declared", with a next step naming
  `model_providers.<id>.billing`; it is never guessed.

## Additional routes (option B)

These routes are not a built-in provider's default but are declared so a custom provider pointing at them gets the
right basis without setup. Each was read on 2026-10-09.

| Route | Credential | Basis | Source |
| --- | --- | --- | --- |
| `https://api.z.ai/api/coding/paas/v4` | Z.AI API key | subscription (GLM Coding Plan) | [Z.AI: other tools](https://docs.z.ai/scenario-example/develop-tools/others): the Coding Plan uses this endpoint "instead of the General API" |
| `https://open.bigmodel.cn/api/coding/paas/v4` | BigModel API key | subscription (GLM Coding Plan) | [ZCode: connect models](https://zcode.z.ai/en/docs/configuration) |
| `https://open.bigmodel.cn/api/paas/v4` | BigModel API key | pay per use | Same page: resource packages and prepaid balance use the general URL |
| `https://api.moonshot.ai/v1` | Kimi Open Platform key | pay per use | [Kimi API overview](https://platform.kimi.ai/docs/api/overview); the membership and the platform are separate products (troubleshooting page above) |
| `https://api.openai.com/v1` | OpenAI API key | pay per use | [Codex pricing](https://developers.openai.com/codex/pricing) |
