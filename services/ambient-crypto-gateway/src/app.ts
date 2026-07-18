import { randomUUID } from "node:crypto";

import express from "express";
import type { NextFunction, Request, RequestHandler, Response } from "express";
import { parseSIWxHeader } from "@x402/extensions/sign-in-with-x";

import { PLAN_IDS, PLANS, PLAN_MODELS, purchasePath } from "./plans.js";
import type { GatewayStore, PlanLimitReached, SubscriptionPeriod, UsageReservation } from "./store.js";
import { hashToken } from "./token.js";
import { estimateRequestUsage, extractActualUsage, StreamUsageParser } from "./usage.js";
import { createWalletChallenge, verifyWalletChallenge } from "./wallet-auth.js";

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
  publicBaseUrl?: string;
  paymentNetwork?: string;
  paymentAsset?: string;
  paymentReceiver?: string;
  solanaRpcUrl?: string;
}

export function createGatewayApp(options: GatewayAppOptions): express.Express {
  const app = express();
  const now = options.now ?? (() => new Date());
  const fetchImpl = options.fetch ?? globalThis.fetch;
  const ambientBaseUrl = new URL(options.ambientBaseUrl ?? "https://api.ambient.xyz");
  const walletAddressFromRequest = options.walletAddressFromRequest ?? siwxWalletAddress;
  const gatewayOrigin = new URL(options.publicBaseUrl ?? "http://127.0.0.1:4021").origin;

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
  app.get("/v1/plans", (_request, response) => {
    response.setHeader("Cache-Control", "no-store");
    response.json({
      plans: PLAN_IDS.map(planId => PLANS[planId]),
      payment: {
        network: options.paymentNetwork,
        asset: options.paymentAsset,
        payTo: options.paymentReceiver,
        rpcUrl: options.solanaRpcUrl,
      },
    });
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
  app.post("/v1/keys/challenge", accountJson, (request, response) => {
    const walletAddress = typeof request.body?.walletAddress === "string" ? request.body.walletAddress : "";
    try {
      response.setHeader("Cache-Control", "no-store");
      response.json(createWalletChallenge(walletAddress, options.tokenPepper, now()));
    } catch (error) {
      response.status(400).json({ error: error instanceof Error ? error.message : "invalid wallet" });
    }
  });

  app.post("/v1/keys/wallet", accountJson, async (request, response) => {
    const walletAddress = typeof request.body?.walletAddress === "string" ? request.body.walletAddress : "";
    try {
      await verifyWalletChallenge({
        walletAddress,
        challenge: typeof request.body?.challenge === "string" ? request.body.challenge : "",
        signature: typeof request.body?.signature === "string" ? request.body.signature : "",
        gatewayOrigin,
        pepper: options.tokenPepper,
        now: now(),
        consumeNonce: nonce => options.store.hasUsedNonce(nonce),
      });
    } catch (error) {
      response.status(401).json({ error: error instanceof Error ? error.message : "wallet signature is invalid" });
      return;
    }
    try {
      const key = await options.store.createApiKey(walletAddress, now(), options.tokenPepper);
      response.setHeader("Cache-Control", "no-store");
      response.status(201).json(key);
    } catch (error) {
      const message = error instanceof Error ? error.message : "API key creation failed";
      response.status(403).json({ error: message });
    }
  });

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

  app.get("/v1/keys", async (request, response) => {
    const walletAddress = requireWalletAddress(request, response, walletAddressFromRequest);
    if (!walletAddress) return;
    response.setHeader("Cache-Control", "no-store");
    response.json({ keys: await options.store.listApiKeys(walletAddress) });
  });

  const revokeKey = async (request: Request, response: Response) => {
    const walletAddress = requireWalletAddress(request, response, walletAddressFromRequest);
    if (!walletAddress) return;
    const keyId = typeof request.params.keyId === "string"
      ? request.params.keyId
      : typeof request.body?.keyId === "string"
        ? request.body.keyId
        : "";
    if (!UUID_PATTERN.test(keyId)) {
      response.status(400).json({ error: "keyId must be a UUID" });
      return;
    }
    const revoked = await options.store.revokeApiKey(walletAddress, keyId, now());
    response.status(revoked ? 204 : 404).end();
  };
  app.delete("/v1/keys/:keyId", revokeKey);
  app.delete("/v1/keys", accountJson, revokeKey);

  const authenticateApiKey = createApiKeyAuth(options.store, options.tokenPepper, now);
  app.get("/v1/account", authenticateApiKey, async (_request, response) => {
    const account = await options.store.accountForApiKey(response.locals.apiKeyHash as string, now());
    if (!account) {
      response.status(401).json({ error: "API key is invalid, revoked, or expired" });
      return;
    }
    response.setHeader("Cache-Control", "no-store");
    response.json(account);
  });
  app.get("/v1/models", authenticateApiKey, (_request, response) => {
    response.setHeader("Cache-Control", "no-store");
    response.json({ object: "list", data: PLAN_MODELS.map(id => ({ id, object: "model" })) });
  });
  const proxyBody = express.raw({ type: "application/json", limit: MAX_PROXY_BODY_BYTES });
  for (const path of ["/v1/chat/completions", "/v1/messages"]) {
    app.post(path, authenticateApiKey, proxyBody, async (request, response) => {
      let estimatedUsage;
      try {
        estimatedUsage = estimateRequestUsage(request.body);
      } catch (error) {
        const message = error instanceof Error ? error.message : "invalid inference body";
        response.status(400).json({ error: message });
        return;
      }
      const period = response.locals.subscription as SubscriptionPeriod;
      const plan = PLANS[period.planId];
      if (!plan.modelAllowlist.includes(estimatedUsage.model)) {
        response.status(400).json({ error: { type: "model_not_in_plan", model: estimatedUsage.model } });
        return;
      }
      if (estimatedUsage.maxOutputTokens > plan.maxOutputTokens) {
        response.status(400).json({ error: { type: "output_limit_exceeded", maxOutputTokens: plan.maxOutputTokens } });
        return;
      }
      const requestId = clientRequestId(request);
      const authorization = await options.store.reserveApiKeyUsage(
        response.locals.apiKeyHash as string,
        requestId,
        estimatedUsage.model,
        estimatedUsage.reservedTokens,
        now(),
      );
      if (!authorization) {
        response.status(401).json({ error: "API key is invalid, revoked, or expired" });
        return;
      }
      if (authorization.kind === "limit") {
        writeLimitResponse(response, authorization);
        return;
      }
      writePlanHeaders(response, authorization.reservation);
      response.setHeader("X-PfTerminal-Request-Id", requestId);
      await proxyAmbientRequest(
        request,
        response,
        fetchImpl,
        new URL(path, ambientBaseUrl),
        options.ambientApiKey,
        options.store,
        authorization.reservation,
        now,
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
    response.locals.apiKeyHash = hashToken(token, pepper);
    next();
  };
}

async function proxyAmbientRequest(
  request: Request,
  response: Response,
  fetchImpl: typeof globalThis.fetch,
  upstreamUrl: URL,
  ambientApiKey: string,
  store: GatewayStore,
  reservation: UsageReservation,
  now: () => Date,
): Promise<void> {
  const controller = new AbortController();
  let upstreamStarted = false;
  let settled = false;
  const settle = async (
    disposition: "completed" | "rejected" | "ambiguous",
    usage = undefined as ReturnType<typeof extractActualUsage>,
  ) => {
    if (settled) return;
    settled = true;
    const result = await store.settleApiKeyUsage(reservation.id, disposition, usage, now());
    if (result && !response.headersSent) writePlanHeaders(response, result);
  };
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
    upstreamStarted = true;
    response.status(upstream.status);
    for (const header of ["content-type", "x-request-id", "request-id"]) {
      const value = upstream.headers.get(header);
      if (value) response.setHeader(header, value);
    }
    response.setHeader("Cache-Control", "no-store");
    if (!upstream.body) {
      await settle(upstream.ok ? "ambiguous" : "rejected");
      response.end();
      return;
    }
    const contentType = upstream.headers.get("content-type") ?? "";
    if (!contentType.includes("text/event-stream")) {
      const bytes = new Uint8Array(await upstream.arrayBuffer());
      let usage;
      try { usage = extractActualUsage(JSON.parse(Buffer.from(bytes).toString("utf8"))); } catch {}
      await settle(upstream.ok ? (usage ? "completed" : "ambiguous") : "rejected", usage);
      response.end(bytes);
      return;
    }
    const parser = new StreamUsageParser();
    const reader = upstream.body.getReader();
    for (;;) {
      const { done, value } = await reader.read();
      if (done) break;
      parser.push(value);
      if (!response.write(value)) await new Promise<void>(resolve => response.once("drain", resolve));
    }
    const usage = parser.finish();
    await settle(upstream.ok ? (usage ? "completed" : "ambiguous") : "rejected", usage);
    response.end();
  } catch (error) {
    await settle(upstreamStarted ? "ambiguous" : "rejected");
    if (!response.headersSent) {
      response.status(502).json({ error: "Ambient upstream request failed" });
    } else {
      response.destroy(error instanceof Error ? error : undefined);
    }
  }
}

function clientRequestId(request: Request): string {
  const supplied = request.header("x-pfterminal-request-id")?.trim();
  if (supplied && supplied.length <= 128 && /^[A-Za-z0-9_.:-]+$/.test(supplied)) return supplied;
  return randomUUID();
}

function writeLimitResponse(response: Response, limit: PlanLimitReached): void {
  response.status(429).json({
    error: {
      type: "plan_limit_reached",
      window: limit.window,
      limitTokens: limit.limitTokens,
      usedTokens: limit.usedTokens,
      reservedTokens: limit.reservedTokens,
      remainingTokens: limit.remainingTokens,
      resetsAt: limit.resetsAt.toISOString(),
    },
  });
}

function writePlanHeaders(response: Response, usage: UsageReservation): void {
  response.setHeader("X-PfTerminal-Plan", usage.period.planId);
  response.setHeader("X-PfTerminal-Weekly-Remaining-Tokens", String(usage.weeklyRemainingTokens));
  response.setHeader("X-PfTerminal-Weekly-Resets-At", usage.weekly.endsAt.toISOString());
  response.setHeader("X-PfTerminal-Monthly-Remaining-Tokens", String(usage.monthlyRemainingTokens));
  response.setHeader("X-PfTerminal-Monthly-Resets-At", usage.period.endsAt.toISOString());
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
