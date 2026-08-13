import assert from "node:assert/strict";
import { describe, test } from "node:test";

import express from "express";
import type { RequestHandler } from "express";
import request from "supertest";
import { ed25519 } from "@noble/curves/ed25519";
import { base58 } from "@scure/base";

import { createGatewayApp } from "../src/app.js";
import { parsePlanId } from "../src/plans.js";
import { InMemoryGatewayStore } from "../src/store.js";

const NOW = new Date("2026-07-18T12:00:00.000Z");
const PEPPER = "test-pepper-that-is-at-least-thirty-two-characters";
const UPSTREAM_KEY = "ambient-upstream-secret";

function testPaymentMiddleware(store: InMemoryGatewayStore): RequestHandler {
  return async (incoming, response, next) => {
    if (incoming.path.startsWith("/v1/subscriptions/")) {
      if (incoming.header("x-test-payment") !== "settled") {
        response.status(402).json({ error: "payment required" });
        return;
      }
      const planId = parsePlanId(incoming.path.split("/").at(-1) ?? "");
      if (!planId) {
        response.status(400).json({ error: "invalid plan" });
        return;
      }
      await store.recordSettlement({
        transaction: incoming.header("x-test-transaction") ?? "test-transaction",
        walletAddress: incoming.header("x-test-wallet") ?? "wallet-1",
        planId,
        network: "solana:devnet",
        amountAtomic: "1000000",
        settledAt: NOW,
      });
    } else if (
      ["/v1/subscription", "/v1/keys"].includes(incoming.path) &&
      !incoming.header("x-test-wallet")
    ) {
      response.status(401).json({ error: "wallet signature required" });
      return;
    }
    next();
  };
}

function setup(fetchImpl?: typeof globalThis.fetch, upstreamRequestTimeoutMs?: number) {
  const store = new InMemoryGatewayStore();
  const app = createGatewayApp({
    store,
    tokenPepper: PEPPER,
    ambientApiKey: UPSTREAM_KEY,
    ambientBaseUrl: "https://ambient.test",
    paymentMiddleware: testPaymentMiddleware(store),
    walletAddressFromRequest: incoming => incoming.header("x-test-wallet"),
    now: () => new Date(NOW),
    fetch: fetchImpl,
    upstreamRequestTimeoutMs,
  });
  return { app, store };
}

async function buyAndIssueKey(app: express.Express): Promise<{ id: string; key: string }> {
  await request(app)
    .post("/v1/subscriptions/starter")
    .set("x-test-payment", "settled")
    .set("x-test-wallet", "wallet-1")
    .expect(200);
  const response = await request(app)
    .post("/v1/keys")
    .set("x-test-wallet", "wallet-1")
    .expect(201);
  return response.body as { id: string; key: string };
}

describe("Ambient crypto gateway", () => {
  test("issues a key through a replay-safe wallet challenge without browser identity", async () => {
    const store = new InMemoryGatewayStore();
    const privateKey = new Uint8Array(32).fill(9);
    const walletAddress = base58.encode(ed25519.getPublicKey(privateKey));
    await store.recordSettlement({
      transaction: "signed-wallet-settlement",
      walletAddress,
      planId: "starter",
      network: "solana:mainnet",
      amountAtomic: "1000000",
      settledAt: NOW,
    });
    const app = createGatewayApp({
      store,
      tokenPepper: PEPPER,
      ambientApiKey: UPSTREAM_KEY,
      paymentMiddleware: (_incoming, _response, next) => next(),
      publicBaseUrl: "https://plans.pfterminal.test",
      now: () => new Date(NOW),
    });
    const challengeResponse = await request(app)
      .post("/v1/keys/challenge")
      .send({ walletAddress })
      .expect(200);
    const challenge = challengeResponse.body.challenge as string;
    const message = `pfterminal-plan-ownership-v1\nhttps://plans.pfterminal.test\n${challenge}`;
    const signature = base58.encode(ed25519.sign(new TextEncoder().encode(message), privateKey));
    const body = { walletAddress, challenge, signature };
    const issued = await request(app).post("/v1/keys/wallet").send(body).expect(201);
    assert.match(issued.body.key, /^pft_amb_/);
    await request(app).post("/v1/keys/wallet").send(body).expect(401);
  });

  test("does not activate a subscription without verified settlement", async () => {
    const { app } = setup();
    await request(app).post("/v1/subscriptions/starter").expect(402);
    await request(app).post("/v1/keys").set("x-test-wallet", "wallet-1").expect(403);
  });

  test("activates the paying wallet and issues an opaque non-cacheable key", async () => {
    const { app } = setup();
    const key = await buyAndIssueKey(app);
    assert.match(key.key, /^pft_amb_/);
    const status = await request(app)
      .get("/v1/subscription")
      .set("x-test-wallet", "wallet-1")
      .expect(200)
      .expect("cache-control", "no-store");
    assert.equal(status.body.periods.length, 1);
  });

  test("replaces the customer credential and streams only approved upstream headers", async () => {
    let observedAuthorization: string | null = null;
    const fetchImpl: typeof globalThis.fetch = async (_url, init) => {
      observedAuthorization = new Headers(init?.headers).get("authorization");
      return new Response('{"ok":true,"usage":{"prompt_tokens":12,"completion_tokens":5}}', {
        status: 200,
        headers: {
          "content-type": "application/json",
          "set-cookie": "must-not-pass=1",
          "x-request-id": "upstream-request-1",
        },
      });
    };
    const { app } = setup(fetchImpl);
    const issued = await buyAndIssueKey(app);
    const response = await request(app)
      .post("/v1/chat/completions")
      .set("Authorization", `Bearer ${issued.key}`)
      .set("X-Corbanu-Request-Id", "corbanu-client-request-1")
      .send({ model: "z-ai/glm-5.2", messages: [{ role: "user", content: "hello" }] })
      .expect(200);

    assert.equal(observedAuthorization, `Bearer ${UPSTREAM_KEY}`);
    assert.equal(response.headers["set-cookie"], undefined);
    assert.equal(response.headers["x-request-id"], "upstream-request-1");
    assert.equal(response.headers["x-pfterminal-plan"], "starter");
    assert.equal(Number(response.headers["x-pfterminal-weekly-remaining-tokens"]), 249_983);
    assert.equal(response.headers["x-corbanu-plan"], "starter");
    assert.equal(Number(response.headers["x-corbanu-weekly-remaining-tokens"]), 249_983);
    assert.equal(response.headers["x-corbanu-request-id"], "corbanu-client-request-1");
    assert.equal(response.headers["x-pfterminal-request-id"], "corbanu-client-request-1");
    assert.deepEqual(response.body, {
      ok: true,
      usage: { prompt_tokens: 12, completion_tokens: 5 },
    });
  });

  test("stops inference before Ambient when the paid period allowance is exhausted", async () => {
    let upstreamCalls = 0;
    const { app } = setup(async () => {
      upstreamCalls += 1;
      return new Response('{"usage":{"prompt_tokens":0,"completion_tokens":32768}}');
    });
    const issued = await buyAndIssueKey(app);
    for (let index = 0; index < 7; index += 1) {
      await request(app)
        .post("/v1/chat/completions")
        .set("Authorization", `Bearer ${issued.key}`)
        .send({ model: "z-ai/glm-5.2", messages: [{ role: "user", content: "bounded request" }], max_tokens: 32_768 })
        .expect(200);
    }
    const limited = await request(app)
      .post("/v1/chat/completions")
      .set("Authorization", `Bearer ${issued.key}`)
      .send({ model: "z-ai/glm-5.2", messages: [{ role: "user", content: "over quota" }], max_tokens: 32_768 })
      .expect(429);
    assert.equal(limited.body.error.type, "plan_limit_reached");
    assert.equal(limited.body.error.provider, "pfterminal-plan");
    assert.equal(limited.body.error.window, "weekly");
    assert.equal(limited.headers["x-codex-active-limit"], "pfterminal");
    assert.equal(limited.headers["x-pfterminal-limit-name"], "PfTerminal Plan weekly tokens");
    assert.equal(limited.headers["x-corbanu-limit-name"], "Corbanu Terminal Plan weekly tokens");
    assert.equal(
      limited.headers["x-corbanu-primary-used-percent"],
      limited.headers["x-pfterminal-primary-used-percent"],
    );
    assert.ok(Number(limited.headers["retry-after"]) > 0);
    assert.equal(upstreamCalls, 7);
  });

  test("does not bill the maximum reservation when successful output omits usage", async () => {
    const responseBody = { choices: [{ message: { content: "small answer" } }] };
    const { app } = setup(async () => new Response(JSON.stringify(responseBody), {
      headers: { "content-type": "application/json" },
    }));
    const issued = await buyAndIssueKey(app);
    const inference = await request(app)
      .post("/v1/chat/completions")
      .set("Authorization", `Bearer ${issued.key}`)
      .send({ model: "z-ai/glm-5.2", messages: [{ role: "user", content: "hello" }], max_tokens: 32_768 })
      .expect(200);
    assert.deepEqual(inference.body, responseBody);
    const account = await request(app)
      .get("/v1/account")
      .set("Authorization", `Bearer ${issued.key}`)
      .expect(200);
    assert.ok(account.body.period.monthlyUsedTokens > 0);
    assert.ok(account.body.period.monthlyUsedTokens < 100);
    assert.equal(account.body.period.monthlyReservedTokens, 0);
  });

  test("settles an SSE response when transport errors after the completion marker", async () => {
    const encoder = new TextEncoder();
    let deliveredCompletion = false;
    const stream = new ReadableStream<Uint8Array>({
      pull(controller) {
        if (!deliveredCompletion) {
          deliveredCompletion = true;
          controller.enqueue(encoder.encode(
            'data: {"choices":[{"delta":{"content":"completed answer"}}]}\n\ndata: [DONE]\n\n',
          ));
          return;
        }
        controller.error(new Error("transport closed after protocol completion"));
      },
    });
    const { app } = setup(async () => new Response(stream, {
      status: 200,
      headers: { "content-type": "text/event-stream" },
    }));
    const issued = await buyAndIssueKey(app);

    await request(app)
      .post("/v1/chat/completions")
      .set("Authorization", `Bearer ${issued.key}`)
      .send({ model: "z-ai/glm-5.2", messages: [{ role: "user", content: "hello" }] })
      .expect(200);

    const account = await request(app)
      .get("/v1/account")
      .set("Authorization", `Bearer ${issued.key}`)
      .expect(200);
    assert.ok(account.body.period.monthlyUsedTokens > 0);
    assert.equal(account.body.period.monthlyReservedTokens, 0);
  });

  test("bounds an unfinished upstream stream and conservatively settles its reservation", async () => {
    const fetchImpl: typeof globalThis.fetch = async (_input, init) => {
      const stream = new ReadableStream<Uint8Array>({
        start(controller) {
          init?.signal?.addEventListener("abort", () => {
            controller.error(new Error("test upstream timed out"));
          });
        },
      });
      return new Response(stream, {
        status: 200,
        headers: { "content-type": "text/event-stream" },
      });
    };
    const { app } = setup(fetchImpl, 20);
    const issued = await buyAndIssueKey(app);

    await request(app)
      .post("/v1/chat/completions")
      .set("Authorization", `Bearer ${issued.key}`)
      .send({ model: "z-ai/glm-5.2", messages: [{ role: "user", content: "hello" }] })
      .expect(502);

    const account = await request(app)
      .get("/v1/account")
      .set("Authorization", `Bearer ${issued.key}`)
      .expect(200);
    assert.equal(account.body.period.monthlyReservedTokens, 0);
    assert.ok(account.body.period.monthlyUsedTokens > 0);
  });

  test("rejects unknown and revoked keys before reaching Ambient", async () => {
    let upstreamCalls = 0;
    const fetchImpl: typeof globalThis.fetch = async () => {
      upstreamCalls += 1;
      return new Response("{}");
    };
    const { app } = setup(fetchImpl);
    await request(app)
      .post("/v1/chat/completions")
      .set("Authorization", "Bearer invalid")
      .send({})
      .expect(401);
    const issued = await buyAndIssueKey(app);
    await request(app)
      .delete(`/v1/keys/${issued.id}`)
      .set("x-test-wallet", "wallet-1")
      .expect(204);
    await request(app)
      .post("/v1/chat/completions")
      .set("Authorization", `Bearer ${issued.key}`)
      .send({})
      .expect(401);
    assert.equal(upstreamCalls, 0);
  });

  test("bounds repeated API-key authentication failures per client", async () => {
    let upstreamCalls = 0;
    const { app } = setup(async () => {
      upstreamCalls += 1;
      return new Response("{}");
    });
    for (let attempt = 0; attempt < 10; attempt += 1) {
      await request(app)
        .post("/v1/chat/completions")
        .set("Authorization", "Bearer invalid")
        .send({})
        .expect(401);
    }
    const blocked = await request(app)
      .post("/v1/chat/completions")
      .set("Authorization", "Bearer invalid")
      .send({})
      .expect(429);
    assert.equal(blocked.body.error.type, "authentication_rate_limited");
    assert.ok(Number(blocked.headers["retry-after"]) > 0);
    assert.equal(upstreamCalls, 0);
  });

  test("never returns the upstream credential when Ambient fails", async () => {
    const fetchImpl: typeof globalThis.fetch = async () => {
      throw new Error(`connection failed with ${UPSTREAM_KEY}`);
    };
    const { app } = setup(fetchImpl);
    const issued = await buyAndIssueKey(app);
    const response = await request(app)
      .post("/v1/chat/completions")
      .set("Authorization", `Bearer ${issued.key}`)
      .send({ model: "z-ai/glm-5.2", messages: [] })
      .expect(502);
    assert.equal(JSON.stringify(response.body).includes(UPSTREAM_KEY), false);
  });

  test("caps inference request bodies", async () => {
    const { app } = setup(async () => new Response("{}"));
    const issued = await buyAndIssueKey(app);
    await request(app)
      .post("/v1/chat/completions")
      .set("Authorization", `Bearer ${issued.key}`)
      .set("Content-Type", "application/json")
      .send(JSON.stringify({ prompt: "x".repeat(2 * 1024 * 1024) }))
      .expect(413);
  });
});
