import { Connection, Commitment } from "@solana/web3.js";
import { requireSolanaEnv } from "../env.js";

export function getSolanaConnection(commitment: Commitment = "confirmed") {
  const { rpcUrl } = requireSolanaEnv();
  return new Connection(rpcUrl, commitment);
}
