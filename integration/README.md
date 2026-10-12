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

Run it on its own — these are TypeScript sources, compiled by `tsc` exactly as the
engine compiles them (`npm i` then `tsc` then `node`):

```bash
npm install @solana/web3.js typescript @types/node
npx tsc                       # the engine has a build step; so does this
node your-entry.js            # import { solanaClient } from "./integration/solana/client.js"
```

Note: running the `.ts` files directly with `node` does **not** work — the import
specifiers end in `.js`, as TypeScript requires, and only a build step (or a
TypeScript-aware runtime) resolves them. That is not a hidden trick, it is how
the engine runs this code in production.

Verified 2026-10-12 (compiled `tsc --strict`, then executed against live devnet):

```
getProgramStatus() -> {"ok":true,"network":"devnet","programId":"Pending Deployment","deployed":false}
getSolanaConnection().getSlot() -> 510089225
getBalance("11111111111111111111111111111111") -> 1
getBalance("not-an-address") -> null          (invalid input never throws, it returns null)
```


Live in production, no setup needed:

- `GET https://api.stygiascore.com/v1/solana/program-status` — beacon status
- `GET https://api.stygiascore.com/v1/solana/account/:address` — any devnet account, read live

The rest of the engine (module catalogue, scoring, certificates, registry) lives
in the private repository. Ask and I will give you access.
