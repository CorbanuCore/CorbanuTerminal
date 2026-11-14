import http.server, sys
class H(http.server.BaseHTTPRequestHandler):
    def _r(self):
        n = int(self.headers.get("content-length") or 0)
        if n: self.rfile.read(n)
        body = b'{"error":{"message":"demo server: invalid credentials"}}'
        self.send_response(401); self.send_header("content-type","application/json")
        self.send_header("content-length", str(len(body))); self.end_headers(); self.wfile.write(body)
    do_GET = do_POST = _r
    def log_message(self, *a): pass
http.server.HTTPServer(("127.0.0.1", int(sys.argv[1])), H).serve_forever()
