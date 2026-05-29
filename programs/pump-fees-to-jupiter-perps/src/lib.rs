use anchor_lang::prelude::*;

declare_id!("F3WS96pF3QCpovpw4hcEr1cvmroNYuEZqKYg9G8n9Sw1");

#[program]
pub mod pump_fees_to_jupiter_perps {
    use super::*;

    pub fn ping(_ctx: Context<Ping>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Ping {}

