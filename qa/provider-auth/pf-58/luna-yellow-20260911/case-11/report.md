# Case 11 — Cancel a stream and continue the same session

Status: **PASSED**

The supplied macOS arm64 candidate accepted a multi-paragraph request, visibly streamed several paragraphs, stopped cleanly when Escape was pressed, and remained usable. The follow-up response was visibly exactly `RECOVERY_OK`; the composer prompt returned afterward.

Observed checks:

- PASS — Multi-paragraph response text was visibly streaming before interruption. Evidence: [02-final-pane.txt](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-11/evidence/02-final-pane.txt)
- PASS — Escape stopped the stream cleanly and displayed the interruption state without a visible panic, exit, stall, duplicate output, or `OutputTextDelta without active item`. Evidence: [02-final-pane.txt](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-11/evidence/02-final-pane.txt)
- PASS — The next response was visibly `RECOVERY_OK`. Evidence: [02-final-pane.txt](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-11/evidence/02-final-pane.txt)
- PASS — The composer prompt returned after the recovery response, indicating the same session remained responsive. Evidence: [02-final-pane.txt](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-11/evidence/02-final-pane.txt)

Actions: [actions.txt](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-11/evidence/actions.txt)

Test-instruction ambiguity: “Ask for several paragraphs” does not specify a subject or exact paragraph count. I used the harmless request “Write several paragraphs about the history of the telescope,” which produced multiple visible paragraphs before interruption. No other ambiguity affected execution.

Limitations: This result is limited to the supplied compiled candidate on macOS arm64, the visible TMUX TUI, and this single live provider response. The initial multi-paragraph request was intentionally interrupted, so its complete answer was not evaluated. No Linux inference or native-only UI claim is made.

Candidate SHA-256: `4a8eba7b10199ea49aee42a720687b63cb4b1b2194c2ce164d1b9f6f88ee510e`
