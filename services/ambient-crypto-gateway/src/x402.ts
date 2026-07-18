import type { RequestHandler } from "express";
import {
  paymentMiddlewareFromHTTPServer,
  x402HTTPResourceServer,
  x402ResourceServer,
} from "@x402/express";
import { HTTPFacilitatorClient } from "@x402/core/server";
import { ExactSvmScheme } from "@x402/svm/exact/server";
import {
  createSIWxResourceServerExtension,
  declareSIWxExtension,
} from "@x402/extensions/sign-in-with-x";

import { PLAN_IDS, PLANS, parsePlanId, purchasePath } from "./plans.js";
import type { GatewayStore } from "./store.js";

export interface X402Config {
  store: GatewayStore;
  network: `${string}:${string}`;
  payTo: string;
  publicBaseUrl: URL;
  facilitatorUrl: string;
  facilitatorBearerToken?: string;
  now?: () => Date;
}

export function createX402Middleware(config: X402Config): RequestHandler {
  const facilitatorClient = new HTTPFacilitatorClient({
    url: config.facilitatorUrl,
    createAuthHeaders: config.facilitatorBearerToken
      ? async () => {
          const authorization = `Bearer ${config.facilitatorBearerToken}`;
          return {
            verify: { Authorization: authorization },
            settle: { Authorization: authorization },
            supported: { Authorization: authorization },
          };
        }
      : undefined,
  });
  const now = config.now ?? (() => new Date());
  const resourceServer = new x402ResourceServer(facilitatorClient)
    .register(config.network, new ExactSvmScheme())
    .registerExtension(createSIWxResourceServerExtension({ storage: config.store }))
    .onAfterSettle(async ({ paymentPayload, requirements, result }) => {
      if (!result.success || !result.payer || !result.transaction) {
        return;
      }
      const resourceUrl = paymentPayload.resource?.url;
      if (!resourceUrl) {
        throw new Error("settled payment did not identify its subscription resource");
      }
      const planId = parsePlanId(new URL(resourceUrl).pathname.split("/").at(-1) ?? "");
      if (!planId) {
        throw new Error("settled payment did not identify a supported plan");
      }
      await config.store.recordSettlement({
        transaction: result.transaction,
        walletAddress: result.payer,
        planId,
        network: result.network,
        amountAtomic: result.amount ?? requirements.amount,
        settledAt: now(),
      });
    });

  const routes = Object.fromEntries([
    ...PLAN_IDS.map(planId => [
      `POST ${purchasePath(planId)}`,
      {
        accepts: {
          scheme: "exact" as const,
          price: PLANS[planId].priceUsd,
          network: config.network,
          payTo: config.payTo,
        },
        description: `One month of PfTerminal Ambient ${planId}`,
        mimeType: "application/json",
        resource: new URL(purchasePath(planId), config.publicBaseUrl).toString(),
      },
    ]),
    ...["GET /v1/subscription", "POST /v1/keys", "DELETE /v1/keys"].map(
      route => [
        route,
        {
          accepts: [] as [],
          description: "Wallet-authenticated PfTerminal account operation",
          mimeType: "application/json",
          resource: new URL(route.slice(route.indexOf(" ") + 1), config.publicBaseUrl).toString(),
          extensions: declareSIWxExtension({
            network: [config.network],
            statement: "Authenticate this PfTerminal account operation",
            expirationSeconds: 300,
          }),
        },
      ],
    ),
  ]);
  const httpServer = new x402HTTPResourceServer(resourceServer, routes);
  return paymentMiddlewareFromHTTPServer(httpServer);
}
