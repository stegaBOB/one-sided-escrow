use anchor_lang::prelude::*;
use crate::program::OneSidedEscrow;
use crate::state::*;
use crate::context::*;
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod one_sided_escrow {
	use super::*;

	pub fn set_authority(ctx: Context<SetAuthority>, authority: Pubkey) -> Result<()> {
		let authority_settings = &mut ctx.accounts.authority_settings;
		authority_settings.address = authority;
		Ok(())
	}

	pub fn create_escrow(ctx: Context<CreateEscrow>, seller: Pubkey) -> Result<()> {
		let escrow = &mut ctx.accounts.escrow;
		escrow.buyer = ctx.accounts.buyer.key();
		escrow.seller = seller;
		Ok(())
	}

	pub fn complete_sale(_ctx: Context<CompleteSale>) -> Result<()> {
		Ok(())
	}

	pub fn refund_buyer(_ctx: Context<RefundBuyer>) -> Result<()> {
		Ok(())
	}

	pub fn authority_override(ctx: Context<AuthorityOverride>, authority_ruling: AuthorityRuling) -> Result<()> {
		let escrow_account = &mut ctx.accounts.escrow;
		let authority_settings = &ctx.accounts.authority_settings;
		let authority = &ctx.accounts.authority;
		if authority.key() != authority_settings.address {
			return err!(EscrowError::AuthorityMismatch);
		}
		let sol_destination = match authority_ruling {
			AuthorityRuling::Buyer => ctx.accounts.buyer.to_account_info(),
			AuthorityRuling::Seller => ctx.accounts.seller.to_account_info(),
		};
		escrow_account.close(sol_destination)?;
		Ok(())
	}
}

#[error_code]
pub enum EscrowError {
	#[msg("Authority Pubkey does not match the AuthoritySettings account.")]
	AuthorityMismatch,
}

pub mod state {
	pub use super::*;

	#[account]
	pub struct AuthoritySettings {
		pub address: Pubkey,
	}

	impl AuthoritySettings {
		pub const PREFIX: &'static str = "authority";
		pub const SIZE: usize =
			8 + // discriminator
					32;  // escrow-manager
	}

	#[account]
	pub struct Escrow {
		pub buyer: Pubkey,
		pub seller: Pubkey,
	}

	impl Escrow {
		pub const PREFIX: &'static str = "escrow";
		pub const SIZE: usize =
			8 + // discriminator
					32 + // buyer
					32;  // seller
	}

	#[derive(AnchorSerialize, AnchorDeserialize)]
	pub enum AuthorityRuling {
		Buyer,
		Seller,

	}
}

pub mod context {
	use super::*;

	#[derive(Accounts)]
	pub struct SetAuthority<'info> {
		#[account(mut)]
		pub payer: Signer<'info>,
		#[account(init_if_needed, payer = payer, space = AuthoritySettings::SIZE, seeds = [AuthoritySettings::PREFIX.as_ref()], bump)]
		pub authority_settings: Account<'info, AuthoritySettings>,
		#[account(constraint = program.programdata_address() ? == Some(program_data.key()))]
		pub program: Program<'info, OneSidedEscrow>,
		#[account(constraint = program_data.upgrade_authority_address == Some(upgrade_authority.key()))]
		pub program_data: Account<'info, ProgramData>,
		pub upgrade_authority: Signer<'info>,
		pub system_program: Program<'info, System>,
	}

	#[derive(Accounts)]
	#[instruction(seller: Pubkey)]
	pub struct CreateEscrow<'info> {
		#[account(mut)]
		pub payer: Signer<'info>,
		pub buyer: Signer<'info>,
		#[account(init, payer = payer, space = Escrow::SIZE, seeds = [Escrow::PREFIX.as_ref(), buyer.key().as_ref(), seller.key().as_ref()], bump)]
		pub escrow: Account<'info, Escrow>,
		pub system_program: Program<'info, System>,
	}

	#[derive(Accounts)]
	pub struct CompleteSale<'info> {
		pub buyer: Signer<'info>,
		/// CHECK: Can be any account info. verified in constraints
		#[account(mut)]
		pub seller: AccountInfo<'info>,
		#[account(mut, close = seller, seeds = [Escrow::PREFIX.as_ref(), buyer.key().as_ref(), seller.key().as_ref()], bump, has_one = buyer, has_one = seller)]
		pub escrow: Account<'info, Escrow>,
	}

	#[derive(Accounts)]
	pub struct RefundBuyer<'info> {
		/// CHECK: Can be any account info. verified in constraints
		#[account(mut)]
		pub buyer: AccountInfo<'info>,
		pub seller: Signer<'info>,
		#[account(mut, close = buyer, seeds = [Escrow::PREFIX.as_ref(), buyer.key().as_ref(), seller.key().as_ref()], bump, has_one = buyer, has_one = seller)]
		pub escrow: Account<'info, Escrow>,
	}

	#[derive(Accounts)]
	pub struct AuthorityOverride<'info> {
		/// CHECK: Can be any account info. verified in constraints
		#[account(mut)]
		pub buyer: AccountInfo<'info>,
		/// CHECK: Can be any account info. verified in constraints
		#[account(mut)]
		pub seller: AccountInfo<'info>,
		#[account(mut, seeds = [Escrow::PREFIX.as_ref(), buyer.key().as_ref(), seller.key().as_ref()], bump, has_one = buyer, has_one = seller)]
		pub escrow: Account<'info, Escrow>,
		/// CHECK: Can be any account info. verified in program logic
		#[account(mut)]
		pub authority: AccountInfo<'info>,
		pub program: Account<'info, ProgramData>,
		#[account(seeds = [AuthoritySettings::PREFIX.as_ref()], bump)]
		pub authority_settings: Account<'info, AuthoritySettings>,
	}
}
