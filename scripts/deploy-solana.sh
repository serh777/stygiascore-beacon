#!/bin/bash
# ===========================================
# Deploy WSEO SEO Beacon to Solana (devnet)
# ===========================================
# Prerequisites:
#   - Rust / Cargo installed
#   - Solana CLI installed
#   - Anchor CLI installed (avm)
#   - Solana keypair configured (~/.config/solana/id.json)
#
# Usage:
#   chmod +x scripts/deploy-solana.sh
#   ./scripts/deploy-solana.sh

set -e

echo "🚀 Deploying WSEO SEO Beacon to Solana..."
echo ""

# 1. Build
echo "📦 Building Solana program..."
anchor build

echo "✅ Build successful"
echo ""

# 2. Get program keypair address
PROGRAM_ID=$(solana address -k target/deploy/seo_beacon-keypair.json)
echo "📝 Program ID: $PROGRAM_ID"

# 3. Update Anchor.toml and lib.rs with real program ID
echo "🔧 Updating program ID in Anchor.toml and lib.rs..."
sed -i "s/seo_beacon = \".*\"/seo_beacon = \"$PROGRAM_ID\"/" Anchor.toml
sed -i "s/declare_id!(\".*\")/declare_id!(\"$PROGRAM_ID\")/" programs/seo-beacon/src/lib.rs

# 4. Rebuild with correct ID
echo "📦 Rebuilding with correct program ID..."
anchor build

# 5. Deploy
echo "🌐 Deploying to Solana devnet..."
anchor deploy --provider.cluster devnet

echo ""
echo "🎉 Deployment complete!"
echo "   Program ID: $PROGRAM_ID"
echo ""
echo "Next steps:"
echo "  1. Set SOLANA_PROGRAM_ID=$PROGRAM_ID in your .env"
echo "  2. Update chain-registry.ts in Repo A with the program ID"
echo "  3. Run: npm run dev (to test backend endpoints)"
