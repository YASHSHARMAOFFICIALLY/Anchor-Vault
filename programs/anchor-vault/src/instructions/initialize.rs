use anchor_lang::prelude::*;
use crate::state::ValutState;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub user:Signer<'info>,

    #[account(
        init,
        payer = user,
        seeds = [b"state",user.key().as_ref()],
        bump,
        space = 8 + ValutState::INIT_SPACE
    )]
    pub vault_state:Account<'info,ValutState>,

    #[account(
        seeds=[b"vault",vault_state.key().as_ref()],
        bump
    )]
    pub vault:SystemAccount<'info>,

    pub system_program:Program<'info,System>
}

impl<'info>Initialize<'info>{
    pub fn initialize(&mut self,bump:&InitializeBumps)->Result<()>{
        self.vault_state.vault_bump = bump.vault;
        self.vault_state.state_bump = bump.vault_state;
        
        Ok(())
    }
}