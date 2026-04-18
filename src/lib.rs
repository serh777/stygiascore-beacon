use anchor_lang::prelude::*;
use anchor_lang::solana_program::keccak;

declare_id!("11111111111111111111111111111111");

#[program]
pub mod seo_beacon {
    use super::*;

    /// Initialize the protocol (admin only, one-time)
    pub fn initialize(
        ctx: Context<Initialize>,
        authority: Pubkey,
        registration_fee: u64,
        zk_merkle_root: [u8; 32],
    ) -> Result<()> {
        let protocol = &mut ctx.accounts.protocol;
        protocol.authority = authority;
        protocol.total_scores = 0;
        protocol.total_sbts_minted = 0;
        protocol.registration_fee = registration_fee;
        protocol.protocol_version = "3.0.0-solana".to_string();
        protocol.zk_merkle_root = zk_merkle_root;
        protocol.accept_token_fees = false;
        protocol.bump = ctx.bumps.protocol;

        msg!("Protocol initialized with ZK support");
        Ok(())
    }

    /// Register a score with ZK (Merkle) proof verification
    pub fn register_score_with_zk(
        ctx: Context<RegisterScoreWithZK>,
        target: String,
        score: u8,
        confidence: u8,
        report_hash: [u8; 32],
        ipfs_cid: String,
        input_type: InputType,
        merkle_proof: Vec<[u8; 32]>,
    ) -> Result<()> {
        require!(score <= 100, ErrorCode::InvalidScore);
        require!(confidence <= 100, ErrorCode::InvalidConfidence);
        require!(target.len() <= 64, ErrorCode::TargetTooLong);
        require!(ipfs_cid.len() <= 128, ErrorCode::CidTooLong);

        let protocol = &ctx.accounts.protocol;

        // Verify Merkle proof against stored root
        let is_valid = verify_merkle_proof(
            &merkle_proof,
            &protocol.zk_merkle_root,
            &report_hash,
            score,
        );
        require!(is_valid, ErrorCode::InvalidZKProof);

        let clock = Clock::get()?;
        let score_account = &mut ctx.accounts.score_account;

        score_account.target = target;
        score_account.score = score;
        score_account.confidence = confidence;
        score_account.report_hash = report_hash;
        score_account.ipfs_cid = ipfs_cid;
        score_account.timestamp = clock.unix_timestamp;
        score_account.verifier = ctx.accounts.verifier.key();
        score_account.input_type = input_type;
        score_account.is_valid = true;
        score_account.has_zk_proof = true;
        score_account.zk_proof_hash = keccak::hash(&merkle_proof_to_bytes(&merkle_proof)).to_bytes();
        score_account.bump = ctx.bumps.score_account;

        // Store ZK proof details
        let zk_proof = &mut ctx.accounts.zk_proof_account;
        zk_proof.report_hash = report_hash;
        zk_proof.merkle_root = protocol.zk_merkle_root;
        zk_proof.verified = true;
        zk_proof.public_score = score as u64;
        zk_proof.timestamp = clock.unix_timestamp;
        zk_proof.bump = ctx.bumps.zk_proof_account;

        // Update protocol stats
        let protocol_mut = &mut ctx.accounts.protocol;
        protocol_mut.total_scores += 1;

        emit!(ScoreRegistered {
            target: score_account.target.clone(),
            score,
            confidence,
            report_hash,
            has_zk_proof: true,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    /// Register a public score (no ZK proof required)
    pub fn register_score_public(
        ctx: Context<RegisterScorePublic>,
        target: String,
        score: u8,
        report_hash: [u8; 32],
        ipfs_cid: String,
    ) -> Result<()> {
        require!(score <= 100, ErrorCode::InvalidScore);
        require!(target.len() <= 64, ErrorCode::TargetTooLong);
        require!(ipfs_cid.len() <= 128, ErrorCode::CidTooLong);

        let clock = Clock::get()?;
        let score_account = &mut ctx.accounts.score_account;

        score_account.target = target;
        score_account.score = score;
        score_account.confidence = 50; // Default for public
        score_account.report_hash = report_hash;
        score_account.ipfs_cid = ipfs_cid;
        score_account.timestamp = clock.unix_timestamp;
        score_account.verifier = ctx.accounts.payer.key();
        score_account.input_type = InputType::Website;
        score_account.is_valid = true;
        score_account.has_zk_proof = false;
        score_account.zk_proof_hash = [0; 32];
        score_account.bump = ctx.bumps.score_account;

        let protocol = &mut ctx.accounts.protocol;
        protocol.total_scores += 1;

        emit!(ScoreRegistered {
            target: score_account.target.clone(),
            score,
            confidence: 50,
            report_hash,
            has_zk_proof: false,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    /// Mint an SBT for high-scoring targets (score >= 70)
    pub fn mint_sbt(
        ctx: Context<MintSBT>,
        report_hash: [u8; 32],
        metadata_uri: String,
    ) -> Result<()> {
        let score_account = &ctx.accounts.score_account;

        require!(score_account.is_valid, ErrorCode::InvalidScore);
        require!(score_account.score >= 70, ErrorCode::ScoreTooLow);
        require!(
            score_account.report_hash == report_hash,
            ErrorCode::ReportMismatch
        );
        require!(metadata_uri.len() <= 128, ErrorCode::CidTooLong);

        let clock = Clock::get()?;
        let grade = get_grade(score_account.score);

        let sbt_account = &mut ctx.accounts.sbt_account;
        sbt_account.target = score_account.target.clone();
        sbt_account.report_hash = report_hash;
        sbt_account.timestamp = clock.unix_timestamp;
        sbt_account.score = score_account.score;
        sbt_account.grade = grade.clone();
        sbt_account.metadata_uri = metadata_uri;
        sbt_account.active = true;
        sbt_account.bump = ctx.bumps.sbt_account;

        let protocol = &mut ctx.accounts.protocol;
        protocol.total_sbts_minted += 1;

        emit!(SBTMinted {
            recipient: ctx.accounts.recipient.key(),
            report_hash,
            score: score_account.score,
            grade,
        });

        Ok(())
    }

    /// Register a cross-chain score relayed from an EVM chain
    pub fn register_cross_chain_score(
        ctx: Context<RegisterCrossChainScore>,
        evm_chain_id: u64,
        evm_tx_hash: [u8; 32],
        target: String,
        score: u8,
        confidence: u8,
        report_hash: [u8; 32],
        _wormhole_vaa: Vec<u8>, // Reserved for Wormhole VAA verification
    ) -> Result<()> {
        require!(score <= 100, ErrorCode::InvalidScore);
        require!(confidence <= 100, ErrorCode::InvalidConfidence);
        require!(target.len() <= 64, ErrorCode::TargetTooLong);

        // NOTE: Wormhole VAA verification is a TODO.
        // For now we trust the oracle signer (authority check via has_one).

        let clock = Clock::get()?;
        let cross_chain = &mut ctx.accounts.cross_chain_score;

        cross_chain.evm_chain_id = evm_chain_id;
        cross_chain.evm_tx_hash = evm_tx_hash;
        cross_chain.target = target;
        cross_chain.score = score;
        cross_chain.confidence = confidence;
        cross_chain.report_hash = report_hash;
        cross_chain.timestamp = clock.unix_timestamp;
        cross_chain.verified = true;
        cross_chain.wormhole_sequence = 0; // Extract from VAA when implemented
        cross_chain.bump = ctx.bumps.cross_chain_score;

        emit!(CrossChainScoreRegistered {
            evm_chain_id,
            target: cross_chain.target.clone(),
            score,
            report_hash,
        });

        Ok(())
    }

    /// Update ZK Merkle root (admin only)
    pub fn update_zk_merkle_root(
        ctx: Context<UpdateZKMerkleRoot>,
        new_root: [u8; 32],
    ) -> Result<()> {
        let protocol = &mut ctx.accounts.protocol;
        protocol.zk_merkle_root = new_root;

        emit!(ZKMerkleRootUpdated {
            new_root,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}

// ========================================
// HELPER FUNCTIONS
// ========================================

fn verify_merkle_proof(
    proof: &[[u8; 32]],
    root: &[u8; 32],
    report_hash: &[u8; 32],
    score: u8,
) -> bool {
    // Leaf = keccak256(abi.encodePacked(reportHash, score))
    let mut leaf_data = Vec::with_capacity(33);
    leaf_data.extend_from_slice(report_hash);
    leaf_data.push(score);

    let mut computed_hash = keccak::hash(&leaf_data).to_bytes();

    for proof_element in proof {
        computed_hash = if computed_hash <= *proof_element {
            let mut data = Vec::with_capacity(64);
            data.extend_from_slice(&computed_hash);
            data.extend_from_slice(proof_element);
            keccak::hash(&data).to_bytes()
        } else {
            let mut data = Vec::with_capacity(64);
            data.extend_from_slice(proof_element);
            data.extend_from_slice(&computed_hash);
            keccak::hash(&data).to_bytes()
        };
    }

    computed_hash == *root
}

fn merkle_proof_to_bytes(proof: &[[u8; 32]]) -> Vec<u8> {
    let mut result = Vec::with_capacity(proof.len() * 32);
    for element in proof {
        result.extend_from_slice(element);
    }
    result
}

fn get_grade(score: u8) -> String {
    if score >= 90 {
        "A+".to_string()
    } else if score >= 80 {
        "A".to_string()
    } else if score >= 70 {
        "B+".to_string()
    } else if score >= 60 {
        "B".to_string()
    } else if score >= 50 {
        "C".to_string()
    } else if score >= 40 {
        "D".to_string()
    } else {
        "F".to_string()
    }
}

// ========================================
// ACCOUNT STRUCTURES
// ========================================

#[account]
pub struct Protocol {
    pub authority: Pubkey,          // 32
    pub total_scores: u64,          // 8
    pub total_sbts_minted: u64,     // 8
    pub registration_fee: u64,      // 8
    pub protocol_version: String,   // 4 + len (max 20)
    pub zk_merkle_root: [u8; 32],   // 32
    pub accept_token_fees: bool,    // 1
    pub bump: u8,                   // 1
}

#[account]
pub struct ScoreAccount {
    pub target: String,             // 4 + len (max 64)
    pub score: u8,                  // 1
    pub confidence: u8,             // 1
    pub report_hash: [u8; 32],      // 32
    pub ipfs_cid: String,           // 4 + len (max 128)
    pub timestamp: i64,             // 8
    pub verifier: Pubkey,           // 32
    pub input_type: InputType,      // 1
    pub is_valid: bool,             // 1
    pub has_zk_proof: bool,         // 1
    pub zk_proof_hash: [u8; 32],    // 32
    pub bump: u8,                   // 1
}

#[account]
pub struct ZKProofAccount {
    pub report_hash: [u8; 32],      // 32
    pub merkle_root: [u8; 32],      // 32
    pub verified: bool,             // 1
    pub public_score: u64,          // 8
    pub timestamp: i64,             // 8
    pub bump: u8,                   // 1
}

#[account]
pub struct SBTAccount {
    pub target: String,             // 4 + len (max 64)
    pub report_hash: [u8; 32],      // 32
    pub timestamp: i64,             // 8
    pub score: u8,                  // 1
    pub grade: String,              // 4 + len (max 10)
    pub metadata_uri: String,       // 4 + len (max 128)
    pub active: bool,               // 1
    pub bump: u8,                   // 1
}

#[account]
pub struct CrossChainScore {
    pub evm_chain_id: u64,          // 8
    pub evm_tx_hash: [u8; 32],      // 32
    pub target: String,             // 4 + len (max 64)
    pub score: u8,                  // 1
    pub confidence: u8,             // 1
    pub report_hash: [u8; 32],      // 32
    pub timestamp: i64,             // 8
    pub verified: bool,             // 1
    pub wormhole_sequence: u64,     // 8
    pub bump: u8,                   // 1
}

// ========================================
// INSTRUCTION CONTEXTS
// ========================================

/// Space: 8 (discriminator) + 32 + 8 + 8 + 8 + (4+20) + 32 + 1 + 1 = 122
const PROTOCOL_SPACE: usize = 8 + 32 + 8 + 8 + 8 + (4 + 20) + 32 + 1 + 1;

/// Space for ScoreAccount: 8 + (4+64) + 1 + 1 + 32 + (4+128) + 8 + 32 + 1 + 1 + 1 + 32 + 1 = 318
const SCORE_SPACE: usize = 8 + (4 + 64) + 1 + 1 + 32 + (4 + 128) + 8 + 32 + 1 + 1 + 1 + 32 + 1;

/// Space for ZKProofAccount: 8 + 32 + 32 + 1 + 8 + 8 + 1 = 90
const ZK_PROOF_SPACE: usize = 8 + 32 + 32 + 1 + 8 + 8 + 1;

/// Space for SBTAccount: 8 + (4+64) + 32 + 8 + 1 + (4+10) + (4+128) + 1 + 1 = 265
const SBT_SPACE: usize = 8 + (4 + 64) + 32 + 8 + 1 + (4 + 10) + (4 + 128) + 1 + 1;

/// Space for CrossChainScore: 8 + 8 + 32 + (4+64) + 1 + 1 + 32 + 8 + 1 + 8 + 1 = 168
const CROSS_CHAIN_SPACE: usize = 8 + 8 + 32 + (4 + 64) + 1 + 1 + 32 + 8 + 1 + 8 + 1;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = PROTOCOL_SPACE,
        seeds = [b"protocol"],
        bump
    )]
    pub protocol: Account<'info, Protocol>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(target: String)]
pub struct RegisterScoreWithZK<'info> {
    #[account(
        mut,
        seeds = [b"protocol"],
        bump = protocol.bump
    )]
    pub protocol: Account<'info, Protocol>,

    #[account(
        init,
        payer = verifier,
        space = SCORE_SPACE,
        seeds = [
            b"score",
            target.as_bytes(),
            verifier.key().as_ref()
        ],
        bump
    )]
    pub score_account: Account<'info, ScoreAccount>,

    #[account(
        init,
        payer = verifier,
        space = ZK_PROOF_SPACE,
        seeds = [
            b"zk_proof",
            target.as_bytes(),
            verifier.key().as_ref()
        ],
        bump
    )]
    pub zk_proof_account: Account<'info, ZKProofAccount>,

    #[account(mut)]
    pub verifier: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(target: String)]
pub struct RegisterScorePublic<'info> {
    #[account(
        mut,
        seeds = [b"protocol"],
        bump = protocol.bump
    )]
    pub protocol: Account<'info, Protocol>,

    #[account(
        init,
        payer = payer,
        space = SCORE_SPACE,
        seeds = [
            b"score",
            b"public",
            target.as_bytes(),
            payer.key().as_ref()
        ],
        bump
    )]
    pub score_account: Account<'info, ScoreAccount>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MintSBT<'info> {
    #[account(
        mut,
        seeds = [b"protocol"],
        bump = protocol.bump
    )]
    pub protocol: Account<'info, Protocol>,

    #[account(
        seeds = [
            b"score",
            score_account.target.as_bytes(),
            score_account.verifier.as_ref()
        ],
        bump = score_account.bump
    )]
    pub score_account: Account<'info, ScoreAccount>,

    #[account(
        init,
        payer = payer,
        space = SBT_SPACE,
        seeds = [
            b"sbt",
            recipient.key().as_ref()
        ],
        bump
    )]
    pub sbt_account: Account<'info, SBTAccount>,

    /// CHECK: recipient is just the target wallet for the SBT
    pub recipient: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(evm_chain_id: u64, evm_tx_hash: [u8; 32], target: String)]
pub struct RegisterCrossChainScore<'info> {
    #[account(
        mut,
        seeds = [b"protocol"],
        bump = protocol.bump,
        has_one = authority
    )]
    pub protocol: Account<'info, Protocol>,

    #[account(
        init,
        payer = authority,
        space = CROSS_CHAIN_SPACE,
        seeds = [
            b"cross_chain",
            &evm_chain_id.to_le_bytes(),
            target.as_bytes()
        ],
        bump
    )]
    pub cross_chain_score: Account<'info, CrossChainScore>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateZKMerkleRoot<'info> {
    #[account(
        mut,
        seeds = [b"protocol"],
        bump = protocol.bump,
        has_one = authority
    )]
    pub protocol: Account<'info, Protocol>,

    pub authority: Signer<'info>,
}

// ========================================
// ENUMS & EVENTS
// ========================================

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum InputType {
    Website,
    Ens,
    Wallet,
    Contract,
    Nft,
}

#[event]
pub struct ScoreRegistered {
    pub target: String,
    pub score: u8,
    pub confidence: u8,
    pub report_hash: [u8; 32],
    pub has_zk_proof: bool,
    pub timestamp: i64,
}

#[event]
pub struct SBTMinted {
    pub recipient: Pubkey,
    pub report_hash: [u8; 32],
    pub score: u8,
    pub grade: String,
}

#[event]
pub struct CrossChainScoreRegistered {
    pub evm_chain_id: u64,
    pub target: String,
    pub score: u8,
    pub report_hash: [u8; 32],
}

#[event]
pub struct ZKMerkleRootUpdated {
    pub new_root: [u8; 32],
    pub timestamp: i64,
}

// ========================================
// ERRORS
// ========================================

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid score value (must be 0-100)")]
    InvalidScore,

    #[msg("Invalid confidence value (must be 0-100)")]
    InvalidConfidence,

    #[msg("Invalid ZK proof")]
    InvalidZKProof,

    #[msg("Score too low for SBT (minimum 70)")]
    ScoreTooLow,

    #[msg("Report hash mismatch")]
    ReportMismatch,

    #[msg("Target string too long (max 64 chars)")]
    TargetTooLong,

    #[msg("IPFS CID too long (max 128 chars)")]
    CidTooLong,
}
