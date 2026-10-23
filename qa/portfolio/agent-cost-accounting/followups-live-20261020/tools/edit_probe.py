"""One-off edit (acc-tart1): probes for the relay's realtime listener on 127.0.0.1:443."""
import sys

path = sys.argv[1] + "/agms-probe"
s = open(path).read()
old = """    tcp_denied("network", "direct TCP to OpenAI IP", ("172.66.0.243", 443))
"""
new = """    tcp_denied("network", "direct TCP to OpenAI IP", ("172.66.0.243", 443))
    tcp_denied("network", "direct TCP to OpenAI IP (relay's pinned upstream)", ("162.159.140.245", 443))
    # acc-tart1: the relay's realtime listener (127.0.0.1:443, api.openai.com pinned there) takes only
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
"""
assert s.count(old) == 1
s = s.replace(old, new)
open(path, "w").write(s)
print("edited")
