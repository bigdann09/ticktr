use anchor_lang::prelude::*;

declare_id!("FoEPCgMFsBCLuopKjM4mzHiQxPqE46oAuMvY6WvgbgN");

#[program]
pub mod ticktr {
    use super::*;

    pub fn setup_manager(ctx: Context<SetupManager>) -> Result<()> {
        ctx.accounts.manager.set_inner(Manager {
            bump: ctx.bumps.manager,
            authority: *ctx.accounts.authority.key,
        });

        msg!("Manager setup complete with authority: {}", manager.authority);
        Ok(())
    }

    pub fn create_event(ctx: Context<CreateEvent>, args: CreateEventArgs) -> Result<()> {
        let mut collection_plugin: Vec<PluginAuthorityPair> = vec![];

        msg!("Event created: {}", event.name);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct SetupManager {
    #[account(
        init,
        payer = authority,
        space = 8 + Manager::INIT_SPACE, // 8 bytes for discriminator, 32 bytes for Pubkey, 1 byte for bump
        seeds = [b"manager"],
        bump
    )]
    pub manager: Account<'info, Manager>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateEvent<'info> {
    pub signer: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut)]
    pub event: Signer<'info>,

    #[account(
        seeds = [b"manager"],
        bump = manager.bump,
        constraint = manager.authority == signer.key() @ anchor_lang::error::ErrorCode::InvalidAuthority
    )]
    pub manager: Account<'info, Manager>,
    pub system_program: Program<'info, System>,

    #[account(address = MPL_CORE_ID)]
    /// Check: This is checked by the address constraint
    pub mpl_core_program: UncheckedAccount<'info>,
}

#[account]
#[derive(InitSpace)]
pub struct Manager {
    pub bump: u8,
    pub authority: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateEventArgs {
    pub name: String,
    pub uri: String,
    pub city: String,
    pub venue: String,
    pub artist: String,
    pub date: String,
    pub time: String,
    pub capacity: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateTicketArgs {
    pub name: String,
    pub uri: String,
    pub hall: String,
    pub section: String,
    pub row: String,
    pub seat: String,
    pub price: u64,
    pub venue_authority: u64,
}