use anchor_lang::prelude::*;

declare_id!("FoEPCgMFsBCLuopKjM4mzHiQxPqE46oAuMvY6WvgbgN");

use mpl_core::{
    accounts::{BaseAssetV1, BaseCollectionV1},
    fetch_external_plugin_adapter_data_info, fetch_plugin,
    instructions::{CreateCollectionV2CpiBuilder, UpdatePluginV1CpiBuilder, CreateV2CpiBuilder, WriteExternalPluginAdapterDataV1CpiBuilder},
    types::{AppDataInitInfo, Attribute, Attributes, ExternalPluginAdapterInitInfo, ExternalPluginAdapterKey, ExternalPluginAdapterSchema, PermanentBurnDelegate, PermanentFreezeDelegate, PermanentTransferDelegate, Plugin, PluginAuthority, PluginAuthorityPair, PluginType, UpdateAuthority},
    ID as MPL_CORE_ID
};

#[program]
pub mod ticktr {
    use super::*;

    pub fn setup_manager(ctx: Context<SetupManager>) -> Result<()> {
        ctx.accounts.manager.set_inner(Manager {
            bump: ctx.bumps.manager,
            authority: *ctx.accounts.authority.key,
        });

        msg!("Manager setup complete with authority: {}", ctx.accounts.manager.authority);
        Ok(())
    }

    pub fn create_event(ctx: Context<CreateEvent>, args: CreateEventArgs) -> Result<()> {
        let mut collection_plugin: Vec<PluginAuthorityPair> = vec![];

        let attribute_list: Vec<Attribute> = vec![
            Attribute {
                key: "City".to_string(),
                value: args.city
            },
            Attribute {
                key: "Venue".to_string(),
                value: args.venue
            },
            Attribute {
                key: "Artist".to_string(),
                value: args.artist
            },
            Attribute {
                key: "Date".to_string(),
                value: args.date
            },
            Attribute {
                key: "Time".to_string(),
                value: args.time
            },
            Attribute {
                key: "Capacity".to_string(),
                value: args.capacity.to_string()
            },
        ];

        collection_plugin.push(PluginAuthorityPair {
            plugin: Plugin::Attributes(Attributes { attribute_list }),
            authority: Some(mpl_core::types::PluginAuthority::UpdateAuthority)
        });

        // create the collection that will hold the tickets
        CreateCollectionV2CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
            .collection(&ctx.accounts.event.to_account_info())
            .update_authority(Some(&ctx.accounts.manager.to_account_info()))
            .payer(&ctx.accounts.payer.to_account_info())
            .system_program(&ctx.accounts.system_program.to_account_info())
            .name(args.name.clone())
            .uri(args.uri)
            .plugins(collection_plugin)
            .invoke()?;

        msg!("Event created: {}", args.name.clone());
        Ok(())
    }

    pub fn create_ticket(ctx: Context<CreateTicket>, args: CreateTicketArgs) -> Result<()> {
        // check that the maximum number of ticket has not been reached yet
        let (_, collection_attribute_list, _) = fetch_plugin::<BaseCollectionV1, Attributes>(
            &ctx.accounts.event.to_account_info(),
            PluginType::Attributes,
        )?;

        // search for the capacity attributes
        let capacity_attribute = collection_attribute_list
            .attribute_list
            .iter()
            .find(|attr| attr.key == "Capacity")
            .ok_or(ErrorCode::MissingAttribute)?;

        // Unwrap the capacity attribute value
        let capacity = capacity_attribute
            .value
            .parse::<u32>()
            .map_err(|_| ErrorCode::NumericOverflow)?;

        require!(
            ctx.accounts.event.num_minted < capacity,
            ErrorCode::MaximumTicketsReached
        );

        let mut ticket_plugin: Vec<PluginAuthorityPair> = vec![];

        // Add an attribute Plugin that will hold the ticket details
        let attribute_list: Vec<Attribute> = vec![
            Attribute {
                key: "Ticket Number".to_string(),
                value: ctx
                    .accounts
                    .event
                    .num_minted
                    .checked_add(1)
                    .ok_or(ErrorCode::NumericOverflow)?
                    .to_string()
            },
            Attribute {
                key: "Hall".to_string(),
                value: args.hall,
            },
            Attribute {
                key: "Section".to_string(),
                value: args.section,
            },
            Attribute {
                key: "Row".to_string(),
                value: args.row,
            },
            Attribute {
                key: "Seat".to_string(),
                value: args.seat,
            },
            Attribute {
                key: "Price".to_string(),
                value: args.price.to_string(),
            }
        ];

        ticket_plugin.push(PluginAuthorityPair {
            plugin: Plugin::Attributes(Attributes { attribute_list }),
            authority: Some(PluginAuthority::UpdateAuthority),
        });

        ticket_plugin.push(PluginAuthorityPair {
            plugin: Plugin::PermanentFreezeDelegate(PermanentFreezeDelegate{ frozen: false }),
            authority: Some(PluginAuthority::UpdateAuthority),
        });

        ticket_plugin.push(PluginAuthorityPair {
            plugin: Plugin::PermanentBurnDelegate(PermanentBurnDelegate{}),
            authority: Some(PluginAuthority::UpdateAuthority),
        });

        ticket_plugin.push(PluginAuthorityPair {
            plugin: Plugin::PermanentTransferDelegate(PermanentTransferDelegate{}),
            authority: Some(PluginAuthority::UpdateAuthority),
        });

        let ticket_external_plugin: Vec<ExternalPluginAdapterInitInfo> = vec![ExternalPluginAdapterInitInfo::AppData(AppDataInitInfo {
            init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
            data_authority: PluginAuthority::Address {
                address: args.venue_authority,
            },
            schema: Some(ExternalPluginAdapterSchema::Binary),
        })];

        let signer_seeds = &[b"manager".as_ref(), &[ctx.accounts.manager.bump]];

        // create ticket
        CreateV2CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
            .asset(&ctx.accounts.ticket.to_account_info())
            .collection(Some(&ctx.accounts.event.to_account_info()))
            .payer(&ctx.accounts.payer.to_account_info())
            .authority(Some(&ctx.accounts.manager.to_account_info()))
            .owner(Some(&ctx.accounts.signer.to_account_info()))
            .system_program(&ctx.accounts.system_program.to_account_info())
            .name(args.name)
            .uri(args.uri)
            .plugins(ticket_plugin)
            .external_plugin_adapters(ticket_external_plugin)
            .invoke_signed(&[signer_seeds])?;

        Ok(())
    }

    pub fn scan_ticket(ctx: Context<ScanTicket>) -> Result<()> {
        let (_, app_data_length) = fetch_external_plugin_adapter_data_info::<BaseAssetV1>(
            &ctx.accounts.ticket.to_account_info(),
            None,
            &ExternalPluginAdapterKey::AppData(PluginAuthority::Address {
                address: ctx.accounts.signer.key(),
            }),
        )?;
        require!(app_data_length == 0, ErrorCode::AlreadyScanned);

        let data: Vec<u8> = "Scanned".as_bytes().to_vec();

        // The venue authority is the `signer` of this instruction
        WriteExternalPluginAdapterDataV1CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
            .asset(&ctx.accounts.ticket.to_account_info())
            .collection(Some(&ctx.accounts.event.to_account_info()))
            .payer(&ctx.accounts.payer.to_account_info())
            .authority(Some(&ctx.accounts.signer.to_account_info()))
            .system_program(&ctx.accounts.system_program.to_account_info())
            .key(ExternalPluginAdapterKey::AppData(
                PluginAuthority::Address {
                    address: ctx.accounts.signer.key()
                },
            ))
            .data(data)
            .invoke()?;

        let signer_seeds = &[b"manager".as_ref()];

        UpdatePluginV1CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
            .asset(&ctx.accounts.ticket.to_account_info())
            .collection(Some(&ctx.accounts.event.to_account_info()))
            .payer(&ctx.accounts.payer.to_account_info())
            .authority(Some(&ctx.accounts.manager.to_account_info()))
            .system_program(&ctx.accounts.system_program.to_account_info())
            .plugin(Plugin::PermanentFreezeDelegate(PermanentFreezeDelegate { frozen: true }))
            .invoke_signed(&[signer_seeds])?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct SetupManager<'info> {
    pub signer: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
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
        constraint = manager.authority == signer.key() @ ErrorCode::InvalidAuthority
    )]
    pub manager: Account<'info, Manager>,
    pub system_program: Program<'info, System>,

    #[account(address = MPL_CORE_ID)]
    /// CHECK: This is checked by the address constraint
    pub mpl_core_program: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct CreateTicket<'info> {
    pub signer: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        seeds = [b"manager"],
        bump = manager.bump
    )]
    pub manager: Account<'info, Manager>,
    #[account(
        mut,
        constraint = event.update_authority == manager.key()
    )]
    pub event: Account<'info, BaseCollectionV1>,
    #[account(mut)]
    pub ticket: Signer<'info>,
    pub system_program: Program<'info, System>,
    /// CHECK: This is checked by the address constraint
    pub mpl_core_program: UncheckedAccount<'info>
}

#[derive(Accounts)]
pub struct ScanTicket<'info> {
    pub owner: Signer<'info>,
    pub signer: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        seeds = [b"manager"],
        bump = manager.bump
    )]
    pub manager: Account<'info, Manager>,
    #[account(
        mut,
        constraint = ticket.owner == owner.key(),
        constraint = ticket.update_authority == UpdateAuthority::Collection(event.key()),
    )]
    pub ticket: Account<'info, BaseAssetV1>,
    #[account(
        mut,
        constraint = event.update_authority == manager.key()
    )]
    pub event: Account<'info, BaseCollectionV1>,
    pub system_program: Program<'info, System>,
    /// CHECK: This is checked by the address constraint
    pub mpl_core_program: UncheckedAccount<'info>
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
    pub venue_authority: Pubkey,
}

#[error_code]
pub enum ErrorCode {
    #[msg("The attribute is missing.")]
    MissingAttribute,
    #[msg("Invalid Authority")]
    InvalidAuthority,
    #[msg("Numeric Overflow")]
    NumericOverflow,
    #[msg("MaximumTicketsReached")]
    MaximumTicketsReached,
    #[msg("Already Scanned")]
    AlreadyScanned
}