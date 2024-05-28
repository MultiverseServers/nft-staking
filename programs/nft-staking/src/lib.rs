use anchor_lang::prelude::*;
use anchor_spl::token;
use anchor_spl::{metadata::Metadata, token::{Mint, Token, TokenAccount, Approve, Revoke}};
use anchor_spl::metadata::mpl_token_metadata::ID as MetadataTokenId;
use anchor_spl::metadata::mpl_token_metadata::instructions::FreezeDelegatedAccount;
use anchor_spl::metadata::mpl_token_metadata::instructions::ThawDelegatedAccount;
use solana_program::program::invoke_signed;

declare_id!("CHPwaCjWbpyLBH8C8GVNaAsGz68AVf7JCWRva71XTAwU");

#[program]
pub mod nft_staking {
    use std::str::FromStr;

    use super::*;

    pub fn stake(ctx: Context<Stake>) -> Result<()> {
        // check stake state
        require!(ctx.accounts.stake_state.stake_state == StakeState::Unstaked,
            StakeError::AlreadyStaked
        );

        // check metadata account
        // let nft_mint = ctx.accounts.nft_mint.key();
        // let metadata_seed = &["metadata".as_bytes(), 
        // ctx.accounts.metadata_program.key.as_ref(), 
        // nft_mint.as_ref()];
        // let (metadata_derived_key, _bump_seed) = Pubkey::find_program_address(metadata_seed, ctx.accounts.metadata_program.key);
        // require!(metadata_derived_key == ctx.accounts.token_metadata_account.key(),
        // StakeError::UninitializedAccount);


        // check is right collection
        let expectecd_creator = Pubkey::from_str("Xfxh5Gd7DCABKJuHPkCe7yNDrT5iXpeSt48LTCC1kcG").unwrap();
        let metadata = mpl_token_metadata::accounts::Metadata::try_from(&ctx.accounts.token_metadata_account).unwrap();
        let creators = metadata.creators.as_ref().unwrap();
        
        require!(creators[0].address == expectecd_creator,
            StakeError::InvalidCollection
        );

        msg!("Stake called");
        // get now time
        let clock = Clock::get().unwrap();

        msg!("Approving delegate");
        let cpi_approve_program = ctx.accounts.token_program.to_account_info();
        let cpi_approve_accounts: Approve<'_> = Approve{
            to: ctx.accounts.nft_token_account.to_account_info(),
            delegate: ctx.accounts.program_authority.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_approve_ctx = CpiContext::new(cpi_approve_program, cpi_approve_accounts);
        token::approve(cpi_approve_ctx, 1)?;

        msg!("freezing token account");
        let authority_bump = ctx.bumps.program_authority;

        let cpi_accounts = FreezeDelegatedAccount{
            delegate: ctx.accounts.program_authority.key(),
            token_account: ctx.accounts.nft_token_account.key(),
            edition: ctx.accounts.nft_edition.key(),
            mint: ctx.accounts.nft_mint.key(),
            token_program: ctx.accounts.token_program.key(),
        };

        // freeze instruction
        let instruction = cpi_accounts.instruction();
        let account_infos = vec![
            ctx.accounts.program_authority.to_account_info(),
            ctx.accounts.nft_token_account.to_account_info(),
            ctx.accounts.nft_edition.to_account_info(),
            ctx.accounts.nft_mint.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
        ];
        let seeds = &["authority".as_bytes(), &[authority_bump]];
        let signers_seeds = &[&seeds[..]];

        // invoke freeze nft
        invoke_signed(&instruction, &account_infos, signers_seeds)?;

        msg!("Saving state");
        ctx.accounts.stake_state.token_account = ctx.accounts.nft_token_account.key();
        ctx.accounts.stake_state.user_pubkey = ctx.accounts.user.key();
        ctx.accounts.stake_state.stake_state = StakeState::Staked;
        ctx.accounts.stake_state.stake_start_time = clock.unix_timestamp;

        msg!("Stake state: {:?}", ctx.accounts.stake_state.stake_state);

        Ok(())
    }

    pub fn unstake(ctx: Context<Unstake>) -> Result<()>{
        // TODO check unstake time
        msg!("Unstake called");
        // Update state
        ctx.accounts.stake_state.stake_state = StakeState::Unstaked;
        msg!("Thawing token");
        let cpi_accounts = ThawDelegatedAccount {
            delegate: ctx.accounts.program_authority.key(),
            token_account: ctx.accounts.nft_token_account.key(),
            edition: ctx.accounts.nft_edition.key(),
            mint: ctx.accounts.nft_mint.key(),
            token_program: ctx.accounts.token_program.key(),
        };
        let instruction = cpi_accounts.instruction();

        let account_infos = vec![
            ctx.accounts.program_authority.to_account_info(),
            ctx.accounts.nft_token_account.to_account_info(),
            ctx.accounts.nft_edition.to_account_info(),
            ctx.accounts.nft_mint.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
        ];

        let seeds = &["authority".as_bytes(), &[ctx.bumps.program_authority]];
        let signers_seeds = &[&seeds[..]];
        invoke_signed(&instruction, &account_infos, signers_seeds)?;

        msg!("Revoking delegate");
        let cpi_revoke_program = ctx.accounts.token_program.to_account_info();
        let cpi_revoke_accounts = Revoke {
            source: ctx.accounts.nft_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };

        let cpi_revoke_ctx = CpiContext::new(cpi_revoke_program, cpi_revoke_accounts);
        token::revoke(cpi_revoke_ctx)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Stake<'info>{
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        associated_token::mint=nft_mint,
        associated_token::authority=user
    )]
    pub nft_token_account: Account<'info, TokenAccount>,
    pub nft_mint: Account<'info, Mint>,
    /// CHECK: Manual validation
    #[account(owner=MetadataTokenId)]
    pub nft_edition: UncheckedAccount<'info>,
    #[account(
        init_if_needed,
        payer=user,
        space = std::mem::size_of::<UserStakeInfo>() + 8,
        seeds = [user.key().as_ref(), nft_token_account.key().as_ref()],
        bump
    )]
    pub stake_state: Account<'info, UserStakeInfo>,
    /// CHECK: Manual validation
    #[account(mut, seeds=["authority".as_bytes().as_ref()], bump)]
    pub program_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub metadata_program: Program<'info, Metadata>,
    /// CHECK: Manual validation
    pub token_metadata_account: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct Unstake<'info>{
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        token::authority=user
    )]
    pub nft_token_account: Account<'info, TokenAccount>,
    pub nft_mint: Account<'info, Mint>,
    /// CHECK: Manual validation
    #[account(owner=MetadataTokenId)]
    pub nft_edition: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [user.key().as_ref(), nft_token_account.key().as_ref()],
        bump,
        constraint = *user.key == stake_state.user_pubkey,
        constraint = nft_token_account.key() == stake_state.token_account
    )]
    pub stake_state: Account<'info, UserStakeInfo>,
    /// CHECK: manual check
    #[account(mut, seeds=["authority".as_bytes().as_ref()], bump)]
    pub program_authority: UncheckedAccount<'info>,
    // Default
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub metadata_program: Program<'info, Metadata>,
}

// Accounts
#[account]
pub struct UserStakeInfo {
    pub token_account: Pubkey,
    pub stake_start_time: i64,
    pub user_pubkey: Pubkey,
    pub stake_state: StakeState,
}

#[derive(Debug, PartialEq, AnchorDeserialize, AnchorSerialize, Clone)]
pub enum StakeState{
    Unstaked,
    Staked,
}

#[error_code]
pub enum StakeError{
    #[msg("NFT already staked")]
    AlreadyStaked,
    #[msg("State account is uninitialized")]
    UninitializedAccount,
    #[msg("Stake state is invalid")]
    InvalidStakeState,
    #[msg("Collection is invalid")]
    InvalidCollection,
}
