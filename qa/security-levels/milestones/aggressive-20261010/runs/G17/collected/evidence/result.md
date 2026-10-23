# G17 executor result

- Run started: 2026-10-11T03:14:42Z
- Preflight verified: coordinator `preflight-summary.json` reports `ok: true`; both supplied component files (`preflight-launcher.json`, `preflight-tmux-child.json`) report `ok: true`; executor `agms-probe all` completed rc=0 and `preflight-executor.json` reports `ok: true`.
- Candidate identity verified: `/opt/agms/pkg/bin/corbanu` SHA-256 `181ad0299d238accbf936512531c90988e8d15cb02a50ea8241f8566b2372a4a`.
- Frozen cases SHA-256 verified: `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4`.
- Scope: assigned cases only, in order AGG-27 then AGG-A01. No human acceptance claimed.

## AGG-27

Outcome: failed

### Starting state / setup

- Launch began at 2026-10-11T03:15:13Z in `$AGMS_RUN/fixture/workspace`.
- The UI first showed the directory-trust choice with `1. Yes, continue` selected; I pressed Enter at 2026-10-11T03:15:19Z. The main screen appeared.
- `/security` at 2026-10-11T03:15:39Z showed `Active in this session: Permissive` (`AGG-27-SECURITY`).
- To establish the required Aggressive state, I moved to Aggressive (2026-10-11T03:15:44Z), confirmed with Enter (2026-10-11T03:15:45Z), saw `Saved: Aggressive (nested agents: refuse)` and the visible restart control, then pressed `r` at 2026-10-11T03:16:25Z. The product restarted.
- `/security` after restart showed `Active in this session: Aggressive` and `Core's level: Aggressive` (`AGG-27-SECURITY-ACTIVE`, 2026-10-11T03:16:43Z).
- No fixture file contents were requested by this case and none were read as test inputs.

### Requested versus actual

1. Requested: on route `zai`, send `Reply with exactly: pong`.  
   Actual: typed the prompt separately at 2026-10-11T03:16:54Z, then pressed Enter separately. The turn completed with response `pong` (`AGG-27-ZAI-PONG-20S`, 2026-10-11T03:17:25Z).
2. Requested: use `/model`, switch to `zai-anthropic` GLM 5.3 Flash, then repeat the same prompt.  
   Actual: `/model` listed GLM 5.3 Flash as current but exposed no visible route/provider switch (`AGG-27-MENU`, 2026-10-11T03:17:30Z). I closed the menu, stopped the session with `agms-tui stop`, and launched through the supplied `agms-tui start --route zai-anthropic` at 2026-10-11T03:17:58Z. The footer showed `Z.AI GLM 5.3 Flash via zai-anthropic default`. The same prompt was typed and Enter was sent separately; it completed with `pong` (`AGG-27-ANTHROPIC-PONG`, 2026-10-11T03:18:12Z). This launcher route switch is a recorded deviation/ambiguity because the `/model` UI did not offer a route choice.
3. Requested: restart and repeat on each route.  
   Actual: restarted through `agms-tui stop` and `agms-tui start --route zai-plain-http` at 2026-10-11T03:18:19Z. Footer showed `Z.AI GLM 5.3 Flash via zaihttp default`. Typed the same prompt separately and pressed Enter separately at 2026-10-11T03:18:21Z. The turn did not complete; the UI displayed the fatal broker error quoted below (`AGG-27-PLAIN-HTTP-PONG`, 2026-10-11T03:19:37Z).

### Positive visible checkpoints

- Route `zai`: `Reply with exactly: pong` followed by response `pong`.
- Route `zai-anthropic`: same response `pong`.
- Aggressive remained the visible active/core level after the required restart.
- Model/provider remained `Z.AI GLM 5.3 Flash` on all three route footers (`zai`, `zai-anthropic`, `zaihttp`).

### Refusal/decline/error text observed verbatim

```
■ Fatal error: the provider base URL `http://127.0.0.1:18443/api/paas/v4` cannot be brokered (it needs HTTPS with a DNS name or IPv4 address and no
query). Security level Aggressive turns broker_model_auth on. To send model requests without the broker, choose Permissive in /security, or set
`broker_model_auth = false` under [features] in config.toml, then restart.
```

This was a product/broker fatal error, not a model refusal.

### Expected results

- **failed** — Every turn completes with `pong`: `zai` and `zai-anthropic` completed with `pong`; `zai-plain-http` did not complete.
- **failed** — No broker or refusal error appears: a fatal broker error appeared on `zai-plain-http`.
- **passed** — Model and provider choices are unchanged by the level: all visible route footers showed `Z.AI GLM 5.3 Flash`, a Z.AI model; route identity remained visible as `zai`, `zai-anthropic`, and `zaihttp`.

VERDICT AGG-27: failed

## AGG-A01

Status: blocked

### Requested versus actual

- No case action was started. The required starting condition is a fresh profile on Permissive. The supplied writable product home had already been changed by the assigned AGG-27 execution, and the provided `agms-tui help` exposes no fresh-profile reset or replacement fixture. To avoid broadening actions or modifying the fixture outside the supplied tools, the case was stopped before launch.

### Checkpoints

- None.

### Expected results

- **blocked** — both turns complete with `pong`: not exercised because the required fresh-profile starting condition could not be supplied with the allowed tools.
- **blocked** — no missing-output-limit/refused request: not exercised for the same reason.

VERDICT AGG-A01: blocked
