export interface SolanaPaymentRequirement {
  network: string;
  amount: bigint;
  asset: string;
  payTo: string;
}

export async function preflightSubscriptionPayment(
  purchaseUrl: URL,
  payerAddress: string,
  rpcUrl: string | undefined,
  fetchImpl: typeof globalThis.fetch = globalThis.fetch,
): Promise<SolanaPaymentRequirement> {
  const challengeResponse = await fetchImpl(purchaseUrl, {
    method: "POST",
    headers: { Accept: "application/json" },
  });
  if (challengeResponse.status !== 402) {
    throw new Error(`payment preflight expected HTTP 402, received ${challengeResponse.status}`);
  }
  const encoded = challengeResponse.headers.get("payment-required");
  if (!encoded) throw new Error("payment preflight response omitted PAYMENT-REQUIRED");
  const decoded = JSON.parse(Buffer.from(encoded, "base64").toString("utf8")) as {
    x402Version?: unknown;
    accepts?: unknown;
  };
  if (decoded.x402Version !== 2 || !Array.isArray(decoded.accepts)) {
    throw new Error("payment preflight response was not an x402 v2 challenge");
  }
  const option = decoded.accepts.find(isSolanaPaymentRequirement);
  if (!option) throw new Error("payment preflight did not offer an exact Solana payment");
  const requirement = {
    network: option.network,
    amount: BigInt(option.amount),
    asset: option.asset,
    payTo: option.payTo,
  };

  const endpoint = rpcUrl || defaultRpcUrl(requirement.network);
  const balance = await readTokenBalance(endpoint, payerAddress, requirement.asset, fetchImpl);
  if (balance < requirement.amount) {
    throw new Error(
      `payer has ${balance} atomic token units but the subscription requires ${requirement.amount}`,
    );
  }
  return requirement;
}

interface PaymentOption {
  scheme: "exact";
  network: string;
  amount: string;
  asset: string;
  payTo: string;
}

function isSolanaPaymentRequirement(value: unknown): value is PaymentOption {
  if (!value || typeof value !== "object") return false;
  const option = value as Partial<PaymentOption>;
  return (
    option.scheme === "exact" &&
    typeof option.network === "string" &&
    option.network.startsWith("solana:") &&
    typeof option.amount === "string" &&
    /^\d+$/.test(option.amount) &&
    typeof option.asset === "string" &&
    typeof option.payTo === "string"
  );
}

async function readTokenBalance(
  rpcUrl: string,
  owner: string,
  mint: string,
  fetchImpl: typeof globalThis.fetch,
): Promise<bigint> {
  const response = await fetchImpl(rpcUrl, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: 1,
      method: "getTokenAccountsByOwner",
      params: [owner, { mint }, { encoding: "jsonParsed", commitment: "confirmed" }],
    }),
  });
  if (!response.ok) throw new Error(`Solana balance preflight failed with HTTP ${response.status}`);
  const body = (await response.json()) as {
    error?: unknown;
    result?: { value?: Array<{ account?: { data?: { parsed?: { info?: { tokenAmount?: { amount?: string } } } } } }> };
  };
  if (body.error || !Array.isArray(body.result?.value)) {
    throw new Error("Solana balance preflight returned an invalid RPC response");
  }
  return body.result.value.reduce((total, account) => {
    const amount = account.account?.data?.parsed?.info?.tokenAmount?.amount;
    return total + (typeof amount === "string" && /^\d+$/.test(amount) ? BigInt(amount) : 0n);
  }, 0n);
}

function defaultRpcUrl(network: string): string {
  return network === "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp"
    ? "https://api.mainnet-beta.solana.com"
    : "https://api.devnet.solana.com";
}
