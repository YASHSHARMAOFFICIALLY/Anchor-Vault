use anchor_lang::prelude::*;


#[derive(InitSpace)]
#[account]
pub struct  ValutState{
    pub vault_bump:u8,
    pub state_bump:u8,
}


