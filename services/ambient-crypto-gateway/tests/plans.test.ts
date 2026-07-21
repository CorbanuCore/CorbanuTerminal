import assert from "node:assert/strict";
import test from "node:test";

import { SOLANA_DEVNET, SOLANA_MAINNET } from "../src/config.js";
import {
  SOLANA_DEVNET_USDC_MINT,
  SOLANA_MAINNET_USDC_MINT,
  solanaUsdcMint,
} from "../src/plans.js";

test("selects the canonical USDC mint for each configured Solana network", () => {
  assert.equal(solanaUsdcMint(SOLANA_MAINNET), SOLANA_MAINNET_USDC_MINT);
  assert.equal(solanaUsdcMint(SOLANA_DEVNET), SOLANA_DEVNET_USDC_MINT);
});
