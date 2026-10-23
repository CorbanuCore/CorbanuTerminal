# Owner decisions for this run (from the coordinator's brief to the integrator, 2026-10-15/16)

- Travis accepted PF-60-S06, S07, S08 and S09 (2026-10-15). S10 may run code-blind on all four before they are
  archived, on the isolated test machine.
- Realtime (#824): if its fix had merged before the freeze it would be included; it has not, so realtime is recorded
  as the tracked known gap (no realtime case).
- Method: a fresh code-blind designer freezes the cases first; an independent reviewer checks them; the design is
  frozen (committed) before the test machine starts.
- The executor runs with GLM 5.3 Flash on Z.AI for functional turns. Providers are reached only via local stand-ins
  unless a real key path is explicitly part of a case (Moonshot real requests are such a path).
- Where a model is offered over several connections, the OpenAI-style connection is the default.
- Moonshot cache read (pending decision D3): only if the owner grants it, ONE second turn per Kimi model (K3, K2.7
  Code) is added to observe a real cache read (under $0.01). Otherwise it is skipped and recorded as "not checked
  live; covered by test". Design it as an optional, separately labelled case.
- Spend: keep real requests minimal.
