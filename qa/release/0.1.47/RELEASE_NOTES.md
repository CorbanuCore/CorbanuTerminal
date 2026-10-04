# Corbanu Terminal 0.1.47

- **Long Claude sessions with many screenshots work again.** Claude rejects a
  request with more than 20 images if any image is wider or taller than 2000
  pixels, so every turn failed, even after "continue". Images are now kept
  within that limit, and existing sessions recover when resumed after updating.
- **Stronger loop protection.** On top of 0.1.46, Corbanu now also stops a model
  that cycles through the same few calls or flips an edit back and forth without
  getting new results.

Restart Corbanu after updating, then resume any session that was failing.
Carries forward the 0.1.44 limitation that cancelling a long streamed answer can
leave later requests stalled; start a fresh session if this happens.

Released under explicit operator authorization. Full cross-platform
qualification and the competitive benchmark cycle remain incomplete.
[Release record](https://github.com/CorbanuCore/CorbanuTerminal/blob/rust-v0.1.47/qa/release/0.1.47/RELEASE.md).
