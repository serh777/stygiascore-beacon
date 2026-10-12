// Minimal shim so the extracted Solana modules compile on their own.
//
// In the private engine this module is a validated environment loader: it
// refuses to start if a required variable is missing or malformed. The
// variable names below are the ones the production deployment uses, so this
// is the same code path we run in production, only without the fat.
export const env = {
  SOLANA_RPC_URL: process.env.SOLANA_RPC_URL ?? "https://api.devnet.solana.com",
  SOLANA_PROGRAM_ID: process.env.SOLANA_PROGRAM_ID,
  SOLANA_RPC_TIMEOUT_MS: Number(process.env.SOLANA_RPC_TIMEOUT_MS ?? 8000),
};

export function requireSolanaEnv() {
  return {
    rpcUrl: env.SOLANA_RPC_URL || "https://api.devnet.solana.com",
    programId: env.SOLANA_PROGRAM_ID,
    timeoutMs: env.SOLANA_RPC_TIMEOUT_MS,
  };
}
