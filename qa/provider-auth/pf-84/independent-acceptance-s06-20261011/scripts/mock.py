#!/usr/bin/env python3
"""Independent S06 acceptance mock (stdlib only, written for this run, not the gate's mocks).

One HTTP server that plays every remote party the candidate may talk to, and logs one JSON line per
request to --log so that "no request was sent" can be checked by counting lines:

* Bedrock Mantle Responses (`*/responses` with SigV4 or bearer auth): replies with the signer
  (SigV4 access key id, region, session-token presence, or which bearer canary was sent).
* AWS credential sources a named account must never touch: IMDS (`/latest/...`), ECS container
  credentials (`/ecs-creds`), STS (`Action=AssumeRole...`), SSO portal (`/federation/credentials`).
  Each returns working fake credentials with a distinctive key id so a fallback would be visible.
* OpenAI issuer: device code (`/api/accounts/deviceauth/*`), `/oauth/token` (code exchange and
  refresh with rotation; reused refresh tokens are rejected), `/oauth/revoke`.
* OpenAI/ChatGPT Responses (`*/responses` with a bearer): replies with the user + token generation or
  API-key canary label; expired access tokens get 401. Rate-limit headers carry a per-user percent.

Control: POST /_ctl {"next_user": "...", "ttl": <access ttl s>, "refresh_ttl": <ttl after refresh>}.
A user written "name~2" is the same ChatGPT person as "name" (same email and user id).
Fake values only. Secret halves of keys and signatures are never logged.
"""
import argparse, base64, hashlib, http.server, json, re, threading, time, urllib.parse

LOCK = threading.Lock()
ST = {"next_user": "work", "ttl": 30, "refresh_ttl": 3600, "gen": {}}
BEARER_CANARIES = {}  # value -> label, from --bearer label=value (fake values only)


def b64(d):
    return base64.urlsafe_b64encode(json.dumps(d).encode()).rstrip(b"=").decode()


def jwt(p):
    return "%s.%s.c2ln" % (b64({"alg": "none", "typ": "JWT"}), b64(p))


def claims(tok):
    try:
        part = tok.split(".")[1]
        return json.loads(base64.urlsafe_b64decode(part + "=" * (-len(part) % 4)))
    except Exception:
        return None


def person(user):
    return user.split("~")[0].split("@")[0]


def tokens(user, gen, ttl):
    now = int(time.time())
    p = person(user)
    ws = "ws-" + (user.split("@ws=")[1] if "@ws=" in user else "main")
    idt = jwt({"sub": "sub-" + p, "email": p + "@example.com", "exp": now + 86400,
               "https://api.openai.com/auth": {"chatgpt_user_id": "cu-" + p, "chatgpt_account_id": ws,
                                               "chatgpt_plan_type": "pro" if p.startswith("pro") else "plus"}})
    return {"id_token": idt, "access_token": jwt({"user": user, "gen": gen, "exp": now + ttl}),
            "refresh_token": "rt.%s.%d" % (user, gen)}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--port", type=int, required=True)
    ap.add_argument("--log", required=True)
    ap.add_argument("--bearer", action="append", default=[])
    a = ap.parse_args()
    for kv in a.bearer:
        k, v = kv.split("=", 1)
        BEARER_CANARIES[v] = k

    def log(**e):
        with LOCK, open(a.log, "a") as f:
            f.write(json.dumps(dict(t=round(time.time(), 3), **e)) + "\n")

    class H(http.server.BaseHTTPRequestHandler):
        protocol_version = "HTTP/1.1"

        def log_message(self, *_):
            pass

        def send(self, code, body=None, ctype="application/json"):
            data = b"" if body is None else (body if isinstance(body, bytes) else json.dumps(body).encode())
            self.send_response(code)
            self.send_header("Content-Type", ctype)
            self.send_header("Content-Length", str(len(data)))
            self.end_headers()
            self.wfile.write(data)

        def ident(self):
            auth = self.headers.get("authorization", "")
            m = re.search(r"Credential=([A-Z0-9]+)/\d+/([a-z0-9-]+)/([a-z0-9-]+)/", auth)
            if m:
                return {"scheme": "sigv4", "akid": m.group(1), "region": m.group(2), "service": m.group(3),
                        "session_token": bool(self.headers.get("x-amz-security-token"))}
            if auth.lower().startswith("bearer "):
                t = auth[7:]
                c = claims(t)
                if c and "user" in c:
                    return {"scheme": "chatgpt", "user": c["user"], "gen": c["gen"], "expired": c["exp"] < time.time()}
                if t in BEARER_CANARIES:
                    return {"scheme": "bearer", "canary": BEARER_CANARIES[t]}
                return {"scheme": "bearer", "canary": "UNKNOWN", "sha8": hashlib.sha256(t.encode()).hexdigest()[:8]}
            return {"scheme": "none"}

        def body(self):
            raw = self.rfile.read(int(self.headers.get("content-length", 0) or 0))
            if self.headers.get("content-encoding"):
                return raw, {}
            try:
                if "json" in self.headers.get("content-type", ""):
                    return raw, json.loads(raw or b"{}")
                return raw, dict(urllib.parse.parse_qsl(raw.decode()))
            except Exception:
                return raw, {}

        def do_PUT(self):  # IMDSv2 token
            self.body()
            log(ev="aws-imds", method="PUT", path=self.path)
            self.send(200, b"imds-token", "text/plain")

        def do_GET(self):
            path = self.path.split("?")[0]
            if path.startswith("/latest/"):
                log(ev="aws-imds", method="GET", path=path)
                if path.endswith("/security-credentials/"):
                    return self.send(200, b"role", "text/plain")
                if "/security-credentials/" in path:
                    return self.send(200, {"Code": "Success", "AccessKeyId": "AKIDIMDSFALLBACK",
                                           "SecretAccessKey": "x", "Token": "t",
                                           "Expiration": "2099-01-01T00:00:00Z"})
                return self.send(200, b"", "text/plain")
            if path.startswith("/ecs-creds"):
                log(ev="aws-ecs", path=path)
                return self.send(200, {"AccessKeyId": "AKIDECSFALLBACK", "SecretAccessKey": "x", "Token": "t",
                                       "Expiration": "2099-01-01T00:00:00Z"})
            if path == "/federation/credentials":
                q = dict(urllib.parse.parse_qsl(urllib.parse.urlparse(self.path).query))
                has_tok = bool(self.headers.get("x-amz-sso_bearer_token"))
                log(ev="aws-sso", role=q.get("role_name"), account=q.get("account_id"), bearer=has_tok)
                return self.send(200, {"roleCredentials": {"accessKeyId": "ASIASSOPROFILE", "secretAccessKey": "x",
                                                           "sessionToken": "sso-session",
                                                           "expiration": int((time.time() + 3600) * 1000)}})
            log(ev="get", path=path, who=self.ident())
            if path.endswith("/models"):
                return self.send(200, {"models": [], "data": []})
            self.send(404)

        def do_POST(self):
            raw, b = self.body()
            path = self.path.split("?")[0]
            if path == "/_ctl":
                ST.update({k: v for k, v in b.items() if k in ("next_user", "ttl", "refresh_ttl")})
                return self.send(200, {})
            if b.get("Action") in ("AssumeRole", "AssumeRoleWithWebIdentity"):
                log(ev="aws-sts", action=b.get("Action"), role=b.get("RoleArn"), who=self.ident())
                x = ("<{a}Response><{a}Result><Credentials><AccessKeyId>ASIAROLECHAIN</AccessKeyId>"
                     "<SecretAccessKey>x</SecretAccessKey><SessionToken>role-session</SessionToken>"
                     "<Expiration>2099-01-01T00:00:00Z</Expiration></Credentials>"
                     "<AssumedRoleUser><Arn>arn:aws:sts::1:assumed-role/r/s</Arn><AssumedRoleId>X:s</AssumedRoleId>"
                     "</AssumedRoleUser></{a}Result></{a}Response>").format(a=b.get("Action"))
                return self.send(200, x.encode(), "text/xml")
            if path.endswith("/deviceauth/usercode"):
                log(ev="device-usercode")
                return self.send(200, {"device_auth_id": "dev-1", "user_code": "MOCK-0001", "interval": "0"})
            if path.endswith("/deviceauth/token"):
                return self.send(200, {"authorization_code": "code." + ST["next_user"], "code_challenge": "c",
                                       "code_verifier": "v"})
            if path == "/oauth/revoke":
                t = b.get("token", "")
                log(ev="revoke", user=t.split(".")[1] if t.startswith("rt.") else ("access" if claims(t) else "?"),
                    kind=b.get("token_type_hint"))
                return self.send(200, {})
            if path == "/oauth/token":
                if b.get("grant_type") == "authorization_code":
                    user = b.get("code", "").split(".", 1)[-1]
                    with LOCK:
                        ST["gen"][user] = 1
                    ttl = -60 if user.startswith("bad") else int(ST["ttl"])
                    log(ev="sign-in", user=user)
                    return self.send(200, tokens(user, 1, ttl))
                rt = b.get("refresh_token", "")
                parts = rt.split(".")
                user, gen = (parts[1], int(parts[2])) if len(parts) == 3 else ("?", -1)
                if user.startswith("bad"):
                    log(ev="refresh-rejected", user=user, why="expired")
                    return self.send(401, {"error": {"code": "refresh_token_expired", "message": "expired"}})
                with LOCK:
                    cur = ST["gen"].get(user)
                    if cur is None or gen != cur:
                        log(ev="refresh-rejected", user=user, why="reused", presented=gen, current=cur)
                        return self.send(401, {"error": {"code": "refresh_token_reused", "message": "reused"}})
                    ST["gen"][user] = cur + 1
                log(ev="refresh", user=user, frm=cur, to=cur + 1)
                return self.send(200, tokens(user, cur + 1, int(ST["refresh_ttl"])))
            if path.endswith("/responses"):
                who = self.ident()
                model = b.get("model")
                if who.get("scheme") == "chatgpt" and who.get("expired"):
                    log(ev="responses-401", path=path, who=who, model=model)
                    return self.send(401, {"error": {"code": "token_expired", "message": "expired access token"}})
                if who.get("scheme") == "bearer" and who.get("canary", "").startswith("fake"):
                    log(ev="responses-401", path=path, who=who, model=model)
                    return self.send(401, {"error": {"code": "invalid_api_key", "message": "Incorrect API key"}})
                log(ev="responses", path=path, who=who, model=model)
                if who["scheme"] == "sigv4":
                    text = "mock: sigv4 %s %s%s" % (who["akid"], who["region"], " +session" if who["session_token"] else "")
                elif who["scheme"] == "chatgpt":
                    text = "mock: chatgpt %s gen%s" % (who["user"], who["gen"])
                else:
                    text = "mock: %s %s" % (who["scheme"], who.get("canary", ""))
                rid = "resp_%d" % int(time.time() * 1000)
                item = {"type": "message", "role": "assistant", "id": "msg_" + rid,
                        "content": [{"type": "output_text", "text": text}]}
                usage = {"input_tokens": 120, "input_tokens_details": {"cached_tokens": 0}, "output_tokens": 9,
                         "output_tokens_details": {"reasoning_tokens": 0}, "total_tokens": 129}
                pct = str(len(who.get("user", "x")) * 7 % 100)
                self.send_response(200)
                self.send_header("Content-Type", "text/event-stream")
                self.send_header("x-codex-primary-used-percent", pct)
                self.send_header("x-codex-primary-window-minutes", "300")
                self.send_header("Connection", "close")
                self.end_headers()
                for name, d in [("response.created", {"type": "response.created", "response": {"id": rid}}),
                                ("response.output_item.done", {"type": "response.output_item.done", "item": item}),
                                ("response.completed", {"type": "response.completed",
                                                        "response": {"id": rid, "usage": usage}})]:
                    self.wfile.write(("event: %s\ndata: %s\n\n" % (name, json.dumps(d))).encode())
                    self.wfile.flush()
                return
            log(ev="post-unhandled", path=path, who=self.ident())
            self.send(404)

    http.server.ThreadingHTTPServer(("127.0.0.1", a.port), H).serve_forever()


if __name__ == "__main__":
    main()
