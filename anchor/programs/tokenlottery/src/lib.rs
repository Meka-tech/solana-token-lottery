#![allow(clippy::result_large_err)]

use core::str;

use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, metadata::Metadata, token_interface::{Mint, mint_to, MintTo, TokenAccount, TokenInterface}};
use anchor_spl::metadata::{
  create_metadata_accounts_v3, 
  CreateMetadataAccountsV3 , 
  CreateMasterEditionV3,
  sign_metadata,
  SignMetadata,
  mpl_token_metadata::types::{CollectionDetails, Creator, DataV2}};

declare_id!("FcKm6qPzP8n96H3EtNo4vWnHL68X9gvbPsE88xrTra8k");

#[constant]
pub const  NAME : &str = "Token Lottery Ticket #";
#[constant]
pub const  SYMBOL : &str = "TLT";
#[constant]
pub const  URI : &str = "https://img.freepik.com/free-photo/yellow-ticket-top-view_1101-121.jpg?semt=ais_hybrid&w=740";

#[program]
pub mod tokenlottery {
    



    use anchor_spl::metadata::create_master_edition_v3;

    use super::*;

    pub fn InitializeConfig(ctx : Context<Initialize> ,  start_time :u64 , end_time:u64 ,ticket_price: u64) -> Result<()>{
        let token_lottery = &mut ctx.accounts.token_lottery;
         token_lottery.bump = ctx.bumps.token_lottery;
        token_lottery.start_time =start_time;
        token_lottery.end_time = end_time;
        token_lottery.ticket_price = ticket_price;
        token_lottery.authority = *ctx.accounts.payer.key;
        token_lottery.lottery_pot_amount = 0;
        token_lottery.total_tickets = 0;
        token_lottery.randomness_account = Pubkey::default();
        token_lottery.winner = 0;
        token_lottery.winner_chosen = false;
        Ok(())
    }

    pub fn InitializeLottery(ctx : Context<InitializeLottery>  ) -> Result<()> {

      let signer_seeds : &[&[&[u8]]]  =  &[&[
        b"collection_mint".as_ref(), &[ctx.bumps.collection_mint]
      ]];

      msg!("Creating Mint Account");
      
      let mint_program = MintTo{
        mint : ctx.accounts.collection_mint.to_account_info(),
        to : ctx.accounts.collection_token_account.to_account_info(),
        authority:ctx.accounts.collection_mint.to_account_info()
      };

      let mint_context  =CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(), mint_program, signer_seeds);

      mint_to( mint_context, 1) ?; 

      msg!("Creating Metadata Account");

      let metadata_accounts = CreateMetadataAccountsV3{
        metadata : ctx.accounts.metadata.to_account_info(),
        mint:ctx.accounts.collection_mint.to_account_info(),
        mint_authority : ctx.accounts.collection_mint.to_account_info(),
        payer : ctx.accounts.payer.to_account_info(),
        update_authority:ctx.accounts.collection_mint.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        rent: ctx.accounts.rent.to_account_info()
      };

      let metadata_context = CpiContext::new_with_signer(ctx.accounts.token_metadata_program.to_account_info(), metadata_accounts, &signer_seeds);
      let data = DataV2{
        name : NAME.to_string(),
        symbol : SYMBOL.to_string(),
        uri : URI.to_string(),
        seller_fee_basis_points : 0,
        creators : Some(vec![Creator{
          address : ctx.accounts.collection_mint.key(),
          verified : false,
          share : 100
        }]),
        collection : None ,
        uses : None

      };

      create_metadata_accounts_v3(
        metadata_context, 
        data, 
        true, 
        true, 
       Some(CollectionDetails::V1 { size: 0 }))?;

       msg!("Creating Master Edition Account");

       let master_edition_accounts = CreateMasterEditionV3{
        payer : ctx.accounts.payer.to_account_info(),
        mint : ctx.accounts.collection_mint.to_account_info(),
        edition :ctx.accounts.master_edition.to_account_info(),
        mint_authority : ctx.accounts.collection_mint.to_account_info(),
        update_authority:ctx.accounts.collection_mint.to_account_info(),
        metadata : ctx.accounts.metadata.to_account_info(),
        token_program:ctx.accounts.token_program.to_account_info(),
        system_program:ctx.accounts.system_program.to_account_info(),
        rent : ctx.accounts.rent.to_account_info()
       };

       let master_edition_context = CpiContext::new_with_signer(ctx.accounts.token_metadata_program.to_account_info(), master_edition_accounts, signer_seeds);

       create_master_edition_v3(master_edition_context, Some(0))?;

       msg!("Verifying Collection");

       let sign_metadata_accounts  = SignMetadata{
        creator : ctx.accounts.collection_mint.to_account_info(),
        metadata : ctx.accounts.metadata.to_account_info(),
       };

      let sign_metadata_context =  CpiContext::new_with_signer(ctx.accounts.token_metadata_program.to_account_info(), sign_metadata_accounts, signer_seeds);
      
       sign_metadata(sign_metadata_context)?;


      Ok(())
    }

}


#[derive(Accounts)]
pub struct Initialize <'info>{

  #[account(mut)]
  pub payer : Signer <'info>,

  #[account(
    init,
    payer = payer,
    space = 8 + TokenLottery::INIT_SPACE,
    seeds = [b"token_lottery".as_ref()],
    bump
  )]
  pub token_lottery : Account<'info ,TokenLottery>,

  pub system_program :  Program<'info , System>
}

#[derive(Accounts)]
pub struct InitializeLottery <'info>{

  #[account(mut)]
  pub payer : Signer<'info >,

  #[account(
    init , 
    payer = payer, 
    mint::decimals = 0, 
    mint::authority = collection_mint, 
    mint::freeze_authority = collection_mint ,
    seeds = [b"collection_mint".as_ref()],
    bump,
  )]
  pub collection_mint : InterfaceAccount<'info , Mint>,

  #[account(
    init , 
    payer = payer ,  
    token::mint = collection_mint , 
    token::authority = collection_token_account , 
    seeds=[b"collection_associcated_token"], 
    bump
  )]
  pub collection_token_account  : InterfaceAccount<'info , TokenAccount>,
    
  #[account(
    mut,
    seeds = [b"metadata" , token_metadata_program.key().as_ref() , collection_mint.key().as_ref()],
    bump,
    seeds::program = token_metadata_program.key()
  )]
  /// CHECK: This account is checked by the metadata smart contract
  pub metadata:  UncheckedAccount<'info>,

  #[account(
    mut,
    seeds = [b"metadata" , token_metadata_program.key().as_ref() , collection_mint.key().as_ref() , b"edition"],
    bump,
    seeds::program = token_metadata_program.key()
  )]
  /// CHECK: This account is checked by the metadata smart contract
  pub master_edition:  UncheckedAccount<'info>,




  pub token_metadata_program: Program<'info, Metadata>,
  pub token_program : Interface<'info ,TokenInterface>,
  pub associated_token_program : Program <'info , AssociatedToken>,
  pub system_program : Program<'info , System>,

  pub rent: Sysvar<'info , Rent>

}


#[account]
#[derive(InitSpace)]
pub struct TokenLottery {
  pub bump : u8,
  pub winner : u64,
  pub winner_chosen :bool,
  pub start_time : u64,
  pub end_time : u64,
  pub lottery_pot_amount : u64,
  pub total_tickets : u64,
  pub ticket_price : u64,
  pub authority : Pubkey,
  pub randomness_account : Pubkey,
}