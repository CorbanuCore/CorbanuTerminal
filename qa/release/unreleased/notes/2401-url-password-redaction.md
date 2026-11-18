## Protected sessions: URL passwords in environment values

- Protected sessions now register every possible URL password in an environment value for redaction, however the password is punctuated (commas, semicolons, quotes, brackets, extra `@` or `:`), including scheme-less proxies and a URL followed by an email address, and redact both the percent-encoded and decoded spellings in later output. Over-redaction is preferred to a missed password. The launch filter is only stricter: values with any `user:password@host` span are now dropped too.
