"""One-off edit (acc-tart1): realtime mode for agms-product, agms-tui and the S09 driver."""
import sys

H = sys.argv[1]


def edit(name, pairs):
    path = H + "/" + name
    s = open(path).read()
    for old, new in pairs:
        assert s.count(old) == 1, (name, old[:60])
        s = s.replace(old, new)
    open(path, "w").write(s)


edit("agms-product", [
    ("""#   agms-product --exec ROUTE MODEL [ARGS...]     `corbanu exec` on the same profile and project
""",
     """#   agms-product --exec ROUTE MODEL [ARGS...]     `corbanu exec` on the same profile and project
#   agms-product --realtime openai MODEL TEXT      acc-tart1: one realtime session through `corbanu app-server`
#                                                  (S09's driver), after an API-key sign-in with the placeholder
"""),
    ("""if [ "${1:-}" = "--exec" ]; then mode=exec; shift; fi
""",
     """if [ "${1:-}" = "--exec" ]; then mode=exec; shift; fi
if [ "${1:-}" = "--realtime" ]; then mode=realtime; shift; fi
"""),
    ("""if [ "$mode" = exec ]; then
""",
     """if [ "$mode" = realtime ]; then
  [ "$route" = openai ] || { echo "realtime runs on the openai route only" >&2; exit 64; }
  # S09's driver README: realtime needs an API-key sign-in on the profile. The key is the synthetic placeholder;
  # the relay injects the real key (D16). Done once per profile.
  if ! grep -q '"OPENAI_API_KEY": *"agms-synthetic' "$RUN/product-home/auth.json" 2>/dev/null; then
    printf '%s\\n' "$P" | env "${env_args[@]}" "$BIN" login --with-api-key -c 'cli_auth_credentials_store="file"' \\
      >> "$RUN/evidence/realtime-sign-in.txt" 2>&1
    echo "[agms] API-key sign-in with the synthetic placeholder (exit $?); output in evidence/realtime-sign-in.txt"
  fi
  exec env "${env_args[@]}" /usr/bin/python3 "$R/bin/agms-realtime-driver" "$BIN" "$RUN/fixture/workspace" \\
    --config 'features.realtime_conversation=true' --config 'cli_auth_credentials_store="file"' \\
    --config "log_dir=\\"$RUN/product-logs\\"" --text "${1:?text}"
fi
if [ "$mode" = exec ]; then
"""),
])

edit("realtime_driver.py", [
    ("""                          [--modality text|audio] [--version v1|v2|v3] [--model M]
\"\"\"""",
     """                          [--modality text|audio] [--version v1|v2|v3] [--model M]
                          [--config KEY=VALUE ...]

acc-tart1 (2026-10-20): --config passes `-c KEY=VALUE` to `corbanu app-server` (the harness turns the
realtime feature on and keeps the profile's file credential store and log folder this way).
\"\"\""""),
    ("""parser.add_argument("--timeout", type=float, default=60)
""",
     """parser.add_argument("--timeout", type=float, default=60)
parser.add_argument("--config", action="append", default=[])
"""),
    ("""    [args.bin, "-C", args.workspace, "app-server"],
""",
     """    [args.bin, "-C", args.workspace, "app-server"] + [a for kv in args.config for a in ("-c", kv)],
"""),
])

edit("agms-tui", [
    ("""  agms-tui config 'key = value'     add one top-level setting to the product profile's config.toml
""",
     """  agms-tui realtime 'text'          run ONE realtime session on the openai route's API key through the
                                    product's app server: start (text output), send the text as speech
                                    input, wait for the reply, stop; prints the reply and the
                                    conversation (thread) ID (the harness refuses sessions over the
                                    owner's limit); output saved as evidence/realtime-N.txt
  agms-tui config 'key = value'     add one top-level setting to the product profile's config.toml
"""),
    ("""    elif cmd == "config":
""",
     """    elif cmd == "realtime":
        text = " ".join(args)
        if not text or "\\n" in text:
            print("give one line of text, e.g. agms-tui realtime 'Reply with the single word OK.'")
            return 64
        argv = [ROOT + "/bin/agms-product", "--realtime", "openai", "gpt-6-luna", text]
        try:
            result = subprocess.run(argv, capture_output=True, text=True, stdin=subprocess.DEVNULL, timeout=300)
            code, out, err = result.returncode, result.stdout, result.stderr
        except subprocess.TimeoutExpired as exc:
            code, out, err = 124, exc.stdout or "", (exc.stderr or "") + "\\nTIMEOUT after 300 s"
            out = out.decode() if isinstance(out, bytes) else out
            err = err.decode() if isinstance(err, bytes) else err
        n = len([1 for _ in open(os.path.join(EVIDENCE, "actions.jsonl"))])
        with open(os.path.join(EVIDENCE, "realtime-%d.txt" % n), "w") as stream:
            stream.write("argv: %s\\nexit: %d\\n--- stdout\\n%s\\n--- stderr\\n%s\\n" % (argv, code, out, err))
        log("actions.jsonl", dict(executed="realtime", argv=argv, exit=code, output="realtime-%d.txt" % n))
        print(out + err)
        print("exit=%d (saved as evidence/realtime-%d.txt)" % (code, n))
        return code
    elif cmd == "config":
"""),
])
print("edited")
