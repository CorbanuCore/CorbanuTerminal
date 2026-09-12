# 5. Select Fable 5.1 Max and make a live request

Run /model, select Claude Fable 5.1 at Max, then send Reply with exactly: FABLE_51_OK. Type the prompt and press Return separately.

Pass: Fable 5.1 is offered, both model labels change immediately, the response is FABLE_51_OK, and no missing-environment or provider-auth-command error appears.

Fail: Fable 5.1 is absent, the header and footer disagree, the request uses another model, or authentication fails.

