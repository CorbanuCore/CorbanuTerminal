import { Pool } from "pg";

import {
  loadAccountingReport,
  renderAccountingReport,
  setWalletClassification,
  type WalletClassification,
} from "./accounting-store.js";
import { readGatewayConfig } from "./config.js";
import { solanaUsdcMint } from "./plans.js";
import { PostgresGatewayStore } from "./postgres-store.js";

const config = readGatewayConfig();
const pool = new Pool({ connectionString: config.databaseUrl });

try {
  await new PostgresGatewayStore(pool).initialize();
  const [command = "report", ...args] = process.argv.slice(2);
  if (command === "report") {
    if (args.length > 0) usage();
    const report = await loadAccountingReport({
      pool,
      receiverAddress: config.payTo,
      usdcMint: solanaUsdcMint(config.network),
      solanaRpcUrl: config.solanaRpcUrl.toString(),
    });
    process.stdout.write(renderAccountingReport(report));
  } else if (command === "classify") {
    const [walletAddress, classificationValue, ...labelParts] = args;
    if (!walletAddress || !classificationValue) usage();
    const classification = parseClassification(classificationValue);
    await setWalletClassification(
      pool,
      walletAddress,
      classification,
      labelParts.join(" "),
    );
    process.stdout.write(`Classified ${walletAddress} as ${classification}.\n`);
  } else {
    usage();
  }
} finally {
  await pool.end();
}

function parseClassification(
  value: string,
): WalletClassification | "unclassified" {
  if (value === "customer" || value === "internal" || value === "unclassified")
    return value;
  usage();
}

function usage(): never {
  throw new Error(
    "usage: accounting report | accounting classify <wallet> <customer|internal|unclassified> [label]",
  );
}
