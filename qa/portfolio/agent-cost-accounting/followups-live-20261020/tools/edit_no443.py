"""One-off edit (acc-tart1): drop the 127.0.0.1:443 realtime listener after the shakedown showed the product's
realtime client goes through HTTPS_PROXY (the relay's CONNECT path) like the Responses WebSocket."""
import sys

H = sys.argv[1]


def edit(name, pairs):
    path = H + "/" + name
    s = open(path).read()
    for old, new in pairs:
        assert s.count(old) == 1, (name, old[:70])
        s = s.replace(old, new)
    open(path, "w").write(s)


edit("provision.sh", [
    ("""# acc-tart1: and OpenAI (a Tart base pins only Z.AI). Executors may also open TCP to 127.0.0.1:443, the relay's
# realtime listener (the product's realtime WebSocket client doesn't use HTTPS_PROXY; api.openai.com -> 127.0.0.1).
""", """# acc-tart1: and OpenAI (a Tart base pins only Z.AI; UTM "macOS 3" had OpenAI in its list already).
"""),
    ("""rt = "pass out quick on lo0 proto tcp from any to 127.0.0.1 port 18443 user { agmx0, agmx1, agmx2, agmx3 } keep state\\n"
assert rt in text
text = text.replace(rt, rt + rt.replace("port 18443", "port 443"), 1)
""", ""),
    ("""grep -q "api.openai.com" /etc/hosts || printf '127.0.0.1 api.openai.com\\n' >> /etc/hosts
""", """grep -q "api.openai.com" /etc/hosts || printf '162.159.140.245 api.openai.com\\n' >> /etc/hosts
"""),
])

edit("setup-agent.sh", [
    ("""# owner's decisions), the relay's realtime listener on 127.0.0.1:443, upstream api.openai.com pinned to its address.""",
     """# owner's decisions), realtime WebSocket sessions capped, upstream api.openai.com pinned to its address."""),
    ("""--realtime-cap $REALTIME_CAP --realtime-port 443 --pin""", """--realtime-cap $REALTIME_CAP --pin"""),
])

edit("agms-probe", [
    ("""    # acc-tart1: the relay's realtime listener (127.0.0.1:443, api.openai.com pinned there) takes only
    # GET /v1/realtime WebSocket upgrades; anything else gets 403.
    for request in ("GET /v1/models ", "POST /v1/responses ", "GET /v1/realtime "):
        name = "realtime listener non-upgrade %s127.0.0.1:443" % request
        try:
            sock = socket.create_connection(("127.0.0.1", 443), timeout=10)
            ctx = ssl.create_default_context(cafile="/opt/agms/etc/mediation-ca.pem")
            tls = ctx.wrap_socket(sock, server_hostname="api.openai.com")
            tls.sendall(("%sHTTP/1.1\\r\\nHost: api.openai.com\\r\\nContent-Length: 0\\r\\n\\r\\n" % request).encode())
            line = tls.recv(200).decode("latin-1").split("\\r\\n", 1)[0]
            tls.close()
            record("network", name, "403", line, " 403 " in line + " ")
        except OSError as exc:
            record("network", name, "403", err_name(exc), False)
""", """    # acc-tart1: the relay forwards /v1/realtime only as a WebSocket upgrade.
    try:
        sock, reply = proxy_connect("api.openai.com:443")
        ctx = ssl.create_default_context(cafile="/opt/agms/etc/mediation-ca.pem")
        tls = ctx.wrap_socket(sock, server_hostname="api.openai.com")
        tls.sendall(b"GET /v1/realtime HTTP/1.1\\r\\nHost: api.openai.com\\r\\nContent-Length: 0\\r\\n\\r\\n")
        line = tls.recv(200).decode("latin-1").split("\\r\\n", 1)[0]
        tls.close()
        record("network", "mediator realtime path without WebSocket upgrade", "403", line, " 403 " in line + " ")
    except OSError as exc:
        record("network", "mediator realtime path without WebSocket upgrade", "403", err_name(exc), False)
"""),
])

edit("acc_mediator.py", [
    ("""- OpenAI realtime (/v1/realtime, WebSocket only). The product's realtime WebSocket client
  connects directly (it doesn't use HTTPS_PROXY), so the guest pins api.openai.com to
  127.0.0.1 and this relay also listens on port 443 (--realtime-port; wildcard bind, loopback
  peers only, en0 inbound dropped by the guest's pf): TLS with the
  same campaign leaf, GET /v1/realtime WebSocket upgrades only, key injected, relayed
  byte for byte. Upstream connections use --pin HOST=IP (api.openai.com's real address).""",
     """- OpenAI realtime (/v1/realtime, WebSocket upgrades only) through CONNECT, key injected,
  relayed byte for byte. Upstream connections use --pin HOST=IP (api.openai.com's address).
- --realtime-port (direct TLS listener for a client that ignores HTTPS_PROXY) exists but is
  not enabled in this run: the shakedown showed the realtime client uses the proxy."""),
])
print("edited")
