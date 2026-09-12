# Case 15 — Check API-key and browser guidance

Result: **PASSED**

The visible `/providers` flow exposed Anthropic’s API-key setup screen. It named `ANTHROPIC_API_KEY`, said the credential is stored in the encrypted vault, and said it is never added to chat. Escape canceled the entry without supplying a key.

The visible OpenAI recovery flow exposed the account-login screen with the device URL, a redacted device code, and guidance to open the link on another device for a remote or headless machine. At the prescribed 40-column width, the explanatory guidance wrapped; ANSI-preserving capture showed the URL enclosed by an OSC 8 terminal hyperlink sequence. Escape canceled the login, and the provider list remained unchanged.

No keychain prompt appeared in the observed screens.

After returning to the empty chat composer, Ctrl+D exited the dedicated target session.

Evidence:

- [API-key entry](</Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-15-replay/evidence/04-anthropic-api-key-entry.txt>)
- [API-key cancellation](</Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-15-replay/evidence/05-after-api-key-cancel.txt>)
- [OpenAI login, redacted](</Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-15-replay/evidence/09-openai-sign-in.txt>)
- [OpenAI login at narrow width](</Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-15-replay/evidence/12-openai-sign-in-narrow.txt>)
- [ANSI hyperlink capture](</Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-15-replay/evidence/13-openai-sign-in-narrow-ansi.txt>)
- [Final provider list after cancellation](</Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-15-replay/evidence/15-final-provider-list.txt>)
- [Normal exit status](</Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-15-replay/evidence/17-exit-status.txt>)

Test-instruction ambiguity: at 40 columns the explanatory text wrapped, but the URL itself fit on one line. Actual mouse activation or browser launch was not performed; clickability is bounded to the visible OSC 8 hyperlink metadata in the terminal capture.
