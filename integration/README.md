# `integration/` — the Solana path of the production engine, extracted

This is the Solana client that the live engine actually runs, copied out of the
private engine repository. It is small on purpose: the RPC calls, the timeout
discipline and the honest `Pending Deployment` answer are the whole Solana
surface of the production API today.

| File | What it does |
|---|---|
| `env.ts` | Shim for the engine's validated environment loader (same variable names, no secrets). |
| `solana/client.ts` | `getBalance`, `getAccountInfo` and `getProgramStatus`, each with a hard timeout so the API never hangs on a slow RPC. `getProgramStatus` is the endpoint that reports `{"programId":"Pending Deployment","deployed":false}` until the beacon is deployed. |
| `solana/connection.ts` | Connection factory with a configurable commitment level. |

Run it on its own:

```bash
npm install @solana/web3.js
export SOLANA_RPC_URL=https://api.devnet.solana.com
# then import { solanaClient } from "./integration/solana/client.js" from your own script
```

Live in production, no setup needed:

- `GET https://api.stygiascore.com/v1/solana/program-status` — beacon status
- `GET https://api.stygiascore.com/v1/solana/account/:address` — any devnet account, read live

The rest of the engine (module catalogue, scoring, certificates, registry) lives
in the private repository. Ask and I will give you access.
