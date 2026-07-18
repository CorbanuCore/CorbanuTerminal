import { Readable } from "node:stream";

import express from "express";
import type { NextFunction, Request, RequestHandler, Response } from "express";
import { parseSIWxHeader } from "@x402/extensions/sign-in-with-x";

import { PLAN_IDS, purchasePath } from "./plans.js";
import type { GatewayStore, SubscriptionPeriod } from "./store.js";
import { hashToken } from "./token.js";

const MAX_PROXY_BODY_BYTES = 2 * 1024 * 1024;
const UUID_PATTERN = /^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;

export interface GatewayAppOptions {
  store: GatewayStore;
  tokenPepper: string;
  ambientApiKey: string;
  ambientBaseUrl?: string;
  paymentMiddleware: RequestHandler;
  walletAddressFromRequest?: (request: Request) => string | undefined;
  now?: () => Date;
  fetch?: typeof globalThis.fetch;
  readiness?: () => Promise<void>;
}

export function createGatewayApp(options: GatewayAppOptions): express.Express {
  const app = express();
  const now = options.now ?? (() => new Date());
  const fetchImpl = options.fetch ?? globalThis.fetch;
  const ambientBaseUrl = new URL(options.ambientBaseUrl ?? "https://api.ambient.xyz");
  const walletAddressFromRequest = options.walletAddressFromRequest ?? siwxWalletAddress;

  app.disable("x-powered-by");
  app.get("/healthz", (_request, response) => response.json({ status: "ok" }));
  app.get("/readyz", async (_request, response) => {
    try {
      await options.readiness?.();
      response.json({ status: "ready" });
    } catch {
      response.status(503).json({ status: "not_ready" });
    }
  });
  app.use(options.paymentMiddleware);

  for (const planId of PLAN_IDS) {
    app.post(purchasePath(planId), (_request, response) => {
      response.setHeader("Cache-Control", "no-store");
      response.json({
        paid: true,
        planId,
        next: "Sign in with the paying wallet and POST /v1/keys.",
      });
    });
  }

  app.get("/v1/subscription", async (request, response) => {
    const walletAddress = requireWalletAddress(request, response, walletAddressFromRequest);
    if (!walletAddress) return;
    const periods = await options.store.listPeriods(walletAddress, now());
    response.setHeader("Cache-Control", "no-store");
    response.json({ walletAddress, periods });
  });

  const accountJson = express.json({ limit: "64kb" });
  app.post("/v1/keys", accountJson, async (request, response) => {
    const walletAddress = requireWalletAddress(request, response, walletAddressFromRequest);
    if (!walletAddress) return;
    try {
      const key = await options.store.createApiKey(walletAddress, now(), options.tokenPepper);
      response.setHeader("Cache-Control", "no-store");
      response.status(201).json(key);
    } catch (error) {
      const message = error instanceof Error ? error.message : "API key creation failed";
      response.status(403).json({ error: message });
    }
  });

  app.delete("/v1/keys", accountJson, async (request, response) => {
    const walletAddress = requireWalletAddress(request, response, walletAddressFromRequest);
    if (!walletAddress) return;
    const keyId = typeof request.body?.keyId === "string" ? request.body.keyId : "";
    if (!UUID_PATTERN.test(keyId)) {
      response.status(400).json({ error: "keyId must be a UUID" });
      return;
    }
    const revoked = await options.store.revokeApiKey(walletAddress, keyId, now());
    response.status(revoked ? 204 : 404).end();
  });

  const authenticateApiKey = createApiKeyAuth(options.store, options.tokenPepper, now);
  const proxyBody = express.raw({ type: "application/json", limit: MAX_PROXY_BODY_BYTES });
  for (const path of ["/v1/chat/completions", "/v1/messages"]) {
    app.post(path, authenticateApiKey, proxyBody, async (request, response) => {
      await proxyAmbientRequest(
        request,
        response,
        fetchImpl,
        new URL(path, ambientBaseUrl),
        options.ambientApiKey,
      );
    });
  }

  app.use((error: unknown, _request: Request, response: Response, _next: NextFunction) => {
    if (
      error instanceof Error &&
      (error.message.includes("request entity too large") ||
        ("type" in error && error.type === "entity.too.large"))
    ) {
      response.status(413).json({ error: "request body exceeds 2 MiB" });
      return;
    }
    response.status(500).json({ error: "gateway request failed" });
  });
  return app;
}

function createApiKeyAuth(
  store: GatewayStore,
  pepper: string,
  now: () => Date,
): RequestHandler {
  return async (request, response, next) => {
    const authorization = request.header("authorization");
    const token = authorization?.startsWith("Bearer ") ? authorization.slice(7) : undefined;
    if (!token || token.length > 256) {
      response.status(401).json({ error: "a valid PfTerminal API key is required" });
      return;
    }
    const period = await store.authenticateApiKey(hashToken(token, pepper), now());
    if (!period) {
      response.status(401).json({ error: "API key is invalid, revoked, or expired" });
      return;
    }
    response.locals.subscription = period satisfies SubscriptionPeriod;
    next();
  };
}

async function proxyAmbientRequest(
  request: Request,
  response: Response,
  fetchImpl: typeof globalThis.fetch,
  upstreamUrl: URL,
  ambientApiKey: string,
): Promise<void> {
  const controller = new AbortController();
  response.on("close", () => {
    if (!response.writableEnded) controller.abort();
  });
  try {
    const upstream = await fetchImpl(upstreamUrl, {
      method: "POST",
      headers: {
        Accept: request.header("accept") ?? "application/json",
        Authorization: `Bearer ${ambientApiKey}`,
        "Content-Type": "application/json",
      },
      body: (request.body as Buffer).toString("utf8"),
      signal: controller.signal,
    });
    response.status(upstream.status);
    for (const header of ["content-type", "x-request-id", "request-id"]) {
      const value = upstream.headers.get(header);
      if (value) response.setHeader(header, value);
    }
    response.setHeader("Cache-Control", "no-store");
    if (!upstream.body) {
      response.end();
      return;
    }
    await new Promise<void>((resolve, reject) => {
      const stream = Readable.fromWeb(upstream.body as never);
      stream.on("error", reject);
      response.on("finish", resolve);
      stream.pipe(response);
    });
  } catch (error) {
    if (!response.headersSent) {
      response.status(502).json({ error: "Ambient upstream request failed" });
    } else {
      response.destroy(error instanceof Error ? error : undefined);
    }
  }
}

function requireWalletAddress(
  request: Request,
  response: Response,
  resolver: (request: Request) => string | undefined,
): string | undefined {
  const walletAddress = resolver(request);
  if (!walletAddress) {
    response.status(401).json({ error: "a valid wallet signature is required" });
  }
  return walletAddress;
}

function siwxWalletAddress(request: Request): string | undefined {
  const header = request.header("sign-in-with-x");
  if (!header) return undefined;
  try {
    return parseSIWxHeader(header).address;
  } catch {
    return undefined;
  }
}
