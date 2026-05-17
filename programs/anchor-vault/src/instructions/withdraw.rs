use crate::{state::ValutState,error::ErrorCode};

use anchor_lang::{
    prelude::*,
    system_program::{transfer,Transfer},
};

#[derive(Accounts)]
pub struct Withdraw<'info>{

    #[account(mut)]
    pub user:Signer<'info>,

    #[account(
        mut,
        seeds=[b"vault",vault_state.key().as_ref()],
        bump = vault_state.vault_bump,
    )]
    pub vault:SystemAccount<'info>,

    #[account(
        seeds = [b"state",user.key().as_ref()],
        bump = vault_state.state_bump
    )]

    pub vault_state:Account<'info,ValutState>,
    system_program:Program<'info,System>,

}

impl<'info>Withdraw<'info>{
    pub fn withdraw(&mut self,amount:u64)->Result<()>{
        require!(amount>0,ErrorCode::InvalidAmount);
        require!(
            amount<= self.vault.lamports(),
            ErrorCode::InsufficientFunds,
        );
        let cpi_account = Transfer{
            from:self.vault.to_account_info(),
            to:self.user.to_account_info(),
        };

        let seeds = &[
            b"vault",
            self.vault_state.to_account_info().key.as_ref(),
            &[self.vault_state.vault_bump],
        ];
        let signer_seeds = &[&seeds[..]];

       let cpi_ctx = CpiContext::new_with_signer(
      System::id(),
      cpi_account,
      signer_seeds,
  );


        transfer(cpi_ctx, amount)?;

        Ok(())
    }
}