#!/usr/bin/env python3
"""cleanexec.py [VAR ...] -- <bin> args...

exec <bin> with a minimal environment (disposable home, no keychain). Each VAR
named before `--` is set from $SECRET, so a real key reaches the process through
its environment only, never argv. Needs S and CORBANU_HOME in the caller's env.
"""
import os, sys

i = sys.argv.index("--")
names, cmd = sys.argv[1:i], sys.argv[i + 1:]
S = os.environ["S"]
env = {
    "HOME": f"{S}/fakehome", "USER": os.environ.get("USER", "qa"), "TERM": "xterm-256color",
    "LANG": "en_US.UTF-8", "PATH": f"{S}/pathbin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin",
    "CORBANU_HOME": os.environ["CORBANU_HOME"], "CORBANU_TEST_NO_NATIVE_KEYRING": "1",
}
for k in ("RUST_LOG", "CORBANU_TMUX_REQUIRED", "HTTPS_PROXY", "SSL_CERT_FILE"):
    if k in os.environ:
        env[k] = os.environ[k]
for n in names:
    env[n] = os.environ["SECRET"]
os.execve(cmd[0], cmd, env)
