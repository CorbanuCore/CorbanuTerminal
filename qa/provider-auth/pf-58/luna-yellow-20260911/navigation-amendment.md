# Pilot navigation aid — 11 September 2026

The user proposed supplying keyboard navigation instructions to the code-blind
executors. A mechanics-only guide was added before dispatching cases 15, 20,
24 and 26. Earlier prompts and reports are preserved unchanged. This is a pilot
amendment, not adoption of a new canonical repository policy.

The guide covers literal text entry followed by a separate Enter, arrows and
menu selection, Escape/cancellation, model-provider tabs, normal exit versus
forced cleanup, and dim placeholder text in terminal captures. Visible UI
instructions take precedence if they differ. Missing options remain findings;
the guide must not supply an alternate route around a failing feature.

Acceptance steps and expected results are unchanged. No source, implementation
rationale, previous result, or proposed fix is disclosed to the executors.
Full prompts are retained in each isolated run's `instructions.md`.

ANSI-preserving TMUX captures are available for inspecting styling. They are
not native screenshots and cannot establish Desktop 3 placement, window overlap,
or native Keychain behavior. A graphical screenshot helper has not yet been
provisioned or verified in the isolated executor environment.
