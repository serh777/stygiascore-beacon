# StygiaScore — Solana beacon and verification evidence

**The score Web3 never had: how discoverable a project is, and how ready it is for the post-quantum era.**

StygiaScore measures two things about a Web3 project that nobody publishes today, and publishes them with the reasons and the receipts:

1. **How discoverable it is** — a PageSpeed-style score built only from free public sources (public RPC, RDAP, DNSSEC/DMARC over DoH, robots/llms.txt, Sourcify, DefiLlama, DexScreener, and our own on-chain registry), with a published ranking that compares the measured score against the market's own ranking.
2. **How exposed it is to the end of today's cryptography** — a post-quantum posture reading, a SHA-512 sealed certificate and, on request, an ML-DSA-65 (FIPS 204) signature that anyone can verify in their own browser.

Try it live (nothing to install, no keys needed):

- Search a target: https://stygiascore.com/lookup/
- Full reading + certificate: https://stygiascore.com/free-score/
- Published ranking: https://stygiascore.com/ranking/
- Public score API: `https://api.stygiascore.com/v1/score/vitalik.eth`

---

## What is in this repository

| Path | What it is |
|---|---|
| `src/lib.rs`, `Cargo.toml`, `Xargo.toml` | The **Solana beacon program** (Anchor/Rust): the on-chain registry of scores. Instructions: `initialize`, `register_score_with_zk`, `register_score_public`, `mint_sbt`, `register_cross_chain_score`, `register_echo_from_score`, `update_zk_merkle_root`. |
| `evidence/nist-acvp/` | The **ACVP evidence**: the dispositions and submitted result files from a demo session we ran with NIST's ACVP demo server, including the SLH-DSA SHA2-128s vector-set results with a **PASS** (test session 763867). It is evidence, not a certification: it records which vectors we ran and how they came back. |

## What is **not** in this repository, and why

The measurement engine, the contracts and the production deployment live in a private repository. That is a deliberate decision, not an accident:

- The product is commercially live, and the engine is the commercial asset.
- The engine's configuration references third-party services and keys that are not mine to publish.
- The public parts are the ones a reviewer can actually check: the Solana program (real code, real commits) and the cryptographic evidence (real NIST session outputs).

**If you are a judge or a reviewer and you need more — the engine, the deployment configuration, a walkthrough, anything — ask and I will give you access and walk you through it live.** I would rather answer questions than publish something I cannot take back.

## Build it

The program builds with the standard Anchor toolchain. It needs **no keys or secrets** of ours; you use your own throwaway Solana devnet keypair, which the Solana CLI generates locally:

```bash
solana-keygen new --outfile ~/.config/solana/devnet.json   # your own throwaway key
solana config set --url devnet --keypair ~/.config/solana/devnet.json
solana airdrop 2                                            # devnet SOL, free

anchor build
anchor test
```

## Deployment status (stated honestly)

- The beacon program is **written and reviewed but not deployed yet**: its `declare_id!` is still the placeholder, and our own production API reports it in its own words — `GET https://api.stygiascore.com/v1/solana/program-status` → `{"ok":true,"network":"devnet","programId":"Pending Deployment","deployed":false}`.
- Solana is already a **target chain inside the engine** (chain id 900): the production API reads Solana devnet accounts today (`/v1/solana/account/:address`).
- Deploying the beacon to devnet, and minting the first Solana SBTs for scoring projects, is the work this repository exists to carry.

## Disclosure

This repository contains work that pre-dates the Colosseum hackathon: the Solana program was written earlier in 2026, and the NIST ACVP evidence is from August 2026. It is published here as the reviewable part of a project that has been under construction since 2024. What was added for Colosseum: Solana support in the score path and the beacon deployment.

## Honest limits

- Nothing here claims NIST certification, and nothing claims to be "quantum-proof". The ACVP evidence records a demo session and a PASS on the vectors we ran; certification is a different process.
- A target that cannot be measured from a server is reported as **not measurable**, with the contract that answered and the block the reading was pinned to. We do not publish invented numbers, and the product's own API will tell you the same thing.
- This repository does not contain audience claims, marketing metrics or testimonials. Every number in the product can be re-run by a third party.

## Licence

Source-available for review. Contact before reuse: the project is a private commercial product.
