## Security

- Update url to 2.5.9 (RUSTSEC-2026-0592: stack overflow in `Url::origin()` on deeply nested `blob:` URLs) and cookie to 0.18.3 (RUSTSEC-2026-0594: private-cookie nonce reuse after `fork()`; Corbanu does not use private cookies). `blob:` URLs whose inner scheme is not http, https or file now get an opaque origin, as the URL Standard requires.
