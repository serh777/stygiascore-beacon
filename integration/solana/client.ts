import { Connection, PublicKey } from "@solana/web3.js";
import { env } from "../env.js";

export class SolanaClient {
  private connection: Connection;
  private programId: string | undefined;

  constructor() {
    const rpcUrl = env.SOLANA_RPC_URL || "https://api.devnet.solana.com";
    this.connection = new Connection(rpcUrl, "confirmed");
    this.programId = env.SOLANA_PROGRAM_ID;
  }

  private async withTimeout<T>(promise: Promise<T>, timeoutMs = env.SOLANA_RPC_TIMEOUT_MS): Promise<T> {
    let timeoutId: NodeJS.Timeout | null = null;
    const timeoutPromise = new Promise<T>((_resolve, reject) => {
      timeoutId = setTimeout(() => reject(new Error("solana_rpc_timeout")), timeoutMs);
    });
    try {
      return await Promise.race([promise, timeoutPromise]);
    } finally {
      if (timeoutId) {
        clearTimeout(timeoutId);
      }
    }
  }

  async getBalance(address: string): Promise<number | null> {
    try {
      const pubKey = new PublicKey(address);
      const balance = await this.withTimeout(this.connection.getBalance(pubKey));
      return balance;
    } catch {
      return null;
    }
  }

  async getAccountInfo(address: string) {
    try {
      const pubKey = new PublicKey(address);
      const info = await this.withTimeout(this.connection.getAccountInfo(pubKey));
      return info;
    } catch {
      return null;
    }
  }

  async getProgramStatus(): Promise<{
    ok: boolean;
    network: string;
    programId: string;
    deployed: boolean;
    reason?: string;
  }> {
    const network = env.SOLANA_RPC_URL
      ? env.SOLANA_RPC_URL.includes("devnet")
        ? "devnet"
        : env.SOLANA_RPC_URL.includes("mainnet")
          ? "mainnet-beta"
          : "custom"
      : "devnet";

    if (!this.programId) {
      return {
        ok: true,
        network,
        programId: "Pending Deployment",
        deployed: false
      };
    }

    try {
      const pubKey = new PublicKey(this.programId);
      const info = await this.withTimeout(this.connection.getAccountInfo(pubKey));
      const deployed = info !== null && info.executable === true;

      return {
        ok: true,
        network,
        programId: this.programId,
        deployed
      };
    } catch (e) {
      const reason = e instanceof Error ? e.message : "solana_rpc_error";
      return {
        ok: false,
        network,
        programId: this.programId,
        deployed: false,
        reason
      };
    }
  }
}

export const solanaClient = new SolanaClient();
