use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    instruction::{AccountMeta, Instruction},
    program::invoke,
};

use crate::constants::*;

pub struct PumpCollectCreatorFeeV2Accounts<'info> {
    pub creator: AccountInfo<'info>,
    pub creator_token_account: AccountInfo<'info>,
    pub creator_vault: AccountInfo<'info>,
    pub creator_vault_token_account: AccountInfo<'info>,
    pub quote_mint: AccountInfo<'info>,
    pub quote_token_program: AccountInfo<'info>,
    pub associated_token_program: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub event_authority: AccountInfo<'info>,
    pub program: AccountInfo<'info>,
}

pub fn pump_collect_creator_fee_v2(accounts: PumpCollectCreatorFeeV2Accounts<'_>) -> Result<()> {
    let ix = Instruction {
        program_id: PUMP_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*accounts.creator.key, false),
            AccountMeta::new(*accounts.creator_token_account.key, false),
            AccountMeta::new(*accounts.creator_vault.key, false),
            AccountMeta::new(*accounts.creator_vault_token_account.key, false),
            AccountMeta::new_readonly(*accounts.quote_mint.key, false),
            AccountMeta::new_readonly(*accounts.quote_token_program.key, false),
            AccountMeta::new_readonly(*accounts.associated_token_program.key, false),
            AccountMeta::new_readonly(*accounts.system_program.key, false),
            AccountMeta::new_readonly(*accounts.event_authority.key, false),
            AccountMeta::new_readonly(*accounts.program.key, false),
        ],
        data: PUMP_COLLECT_CREATOR_FEE_V2_DISCRIMINATOR.to_vec(),
    };

    invoke(
        &ix,
        &[
            accounts.creator,
            accounts.creator_token_account,
            accounts.creator_vault,
            accounts.creator_vault_token_account,
            accounts.quote_mint,
            accounts.quote_token_program,
            accounts.associated_token_program,
            accounts.system_program,
            accounts.event_authority,
            accounts.program,
        ],
    )?;

    Ok(())
}

pub struct PumpAmmCollectCoinCreatorFeeAccounts<'info> {
    pub quote_mint: AccountInfo<'info>,
    pub quote_token_program: AccountInfo<'info>,
    pub coin_creator: AccountInfo<'info>,
    pub coin_creator_vault_authority: AccountInfo<'info>,
    pub coin_creator_vault_ata: AccountInfo<'info>,
    pub coin_creator_token_account: AccountInfo<'info>,
    pub event_authority: AccountInfo<'info>,
    pub program: AccountInfo<'info>,
}

pub fn pump_amm_collect_coin_creator_fee(
    accounts: PumpAmmCollectCoinCreatorFeeAccounts<'_>,
) -> Result<()> {
    let ix = Instruction {
        program_id: PUMP_AMM_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new_readonly(*accounts.quote_mint.key, false),
            AccountMeta::new_readonly(*accounts.quote_token_program.key, false),
            AccountMeta::new_readonly(*accounts.coin_creator.key, false),
            AccountMeta::new_readonly(*accounts.coin_creator_vault_authority.key, false),
            AccountMeta::new(*accounts.coin_creator_vault_ata.key, false),
            AccountMeta::new(*accounts.coin_creator_token_account.key, false),
            AccountMeta::new_readonly(*accounts.event_authority.key, false),
            AccountMeta::new_readonly(*accounts.program.key, false),
        ],
        data: PUMP_AMM_COLLECT_COIN_CREATOR_FEE_DISCRIMINATOR.to_vec(),
    };

    invoke(
        &ix,
        &[
            accounts.quote_mint,
            accounts.quote_token_program,
            accounts.coin_creator,
            accounts.coin_creator_vault_authority,
            accounts.coin_creator_vault_ata,
            accounts.coin_creator_token_account,
            accounts.event_authority,
            accounts.program,
        ],
    )?;

    Ok(())
}

pub struct PumpAmmTransferCreatorFeesToPumpV2Accounts<'info> {
    pub payer: AccountInfo<'info>,
    pub quote_mint: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub associated_token_program: AccountInfo<'info>,
    pub coin_creator: AccountInfo<'info>,
    pub coin_creator_vault_authority: AccountInfo<'info>,
    pub coin_creator_vault_ata: AccountInfo<'info>,
    pub pump_creator_vault: AccountInfo<'info>,
    pub pump_creator_vault_ata: AccountInfo<'info>,
    pub event_authority: AccountInfo<'info>,
    pub program: AccountInfo<'info>,
}

pub fn pump_amm_transfer_creator_fees_to_pump_v2(
    accounts: PumpAmmTransferCreatorFeesToPumpV2Accounts<'_>,
) -> Result<()> {
    let ix = Instruction {
        program_id: PUMP_AMM_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*accounts.payer.key, true),
            AccountMeta::new_readonly(*accounts.quote_mint.key, false),
            AccountMeta::new_readonly(*accounts.token_program.key, false),
            AccountMeta::new_readonly(*accounts.system_program.key, false),
            AccountMeta::new_readonly(*accounts.associated_token_program.key, false),
            AccountMeta::new_readonly(*accounts.coin_creator.key, false),
            AccountMeta::new(*accounts.coin_creator_vault_authority.key, false),
            AccountMeta::new(*accounts.coin_creator_vault_ata.key, false),
            AccountMeta::new(*accounts.pump_creator_vault.key, false),
            AccountMeta::new(*accounts.pump_creator_vault_ata.key, false),
            AccountMeta::new_readonly(*accounts.event_authority.key, false),
            AccountMeta::new_readonly(*accounts.program.key, false),
        ],
        data: PUMP_AMM_TRANSFER_CREATOR_FEES_TO_PUMP_V2_DISCRIMINATOR.to_vec(),
    };

    invoke(
        &ix,
        &[
            accounts.payer,
            accounts.quote_mint,
            accounts.token_program,
            accounts.system_program,
            accounts.associated_token_program,
            accounts.coin_creator,
            accounts.coin_creator_vault_authority,
            accounts.coin_creator_vault_ata,
            accounts.pump_creator_vault,
            accounts.pump_creator_vault_ata,
            accounts.event_authority,
            accounts.program,
        ],
    )?;

    Ok(())
}

pub struct PumpDistributeCreatorFeesV2Accounts<'info> {
    pub payer: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub bonding_curve: AccountInfo<'info>,
    pub sharing_config: AccountInfo<'info>,
    pub creator_vault: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub event_authority: AccountInfo<'info>,
    pub program: AccountInfo<'info>,
    pub creator_vault_quote_token_account: AccountInfo<'info>,
    pub quote_mint: AccountInfo<'info>,
    pub quote_token_program: AccountInfo<'info>,
    pub associated_token_program: AccountInfo<'info>,
}

pub fn pump_distribute_creator_fees_v2<'info>(
    accounts: PumpDistributeCreatorFeesV2Accounts<'info>,
    initialize_ata: bool,
    remaining_accounts: &[AccountInfo<'info>],
) -> Result<()> {
    let data = encode_pump_distribute_creator_fees_v2(initialize_ata);

    let mut metas = vec![
        AccountMeta::new(*accounts.payer.key, true),
        AccountMeta::new_readonly(*accounts.mint.key, false),
        AccountMeta::new_readonly(*accounts.bonding_curve.key, false),
        AccountMeta::new_readonly(*accounts.sharing_config.key, false),
        AccountMeta::new(*accounts.creator_vault.key, false),
        AccountMeta::new_readonly(*accounts.system_program.key, false),
        AccountMeta::new_readonly(*accounts.event_authority.key, false),
        AccountMeta::new_readonly(*accounts.program.key, false),
        AccountMeta::new(*accounts.creator_vault_quote_token_account.key, false),
        AccountMeta::new_readonly(*accounts.quote_mint.key, false),
        AccountMeta::new_readonly(*accounts.quote_token_program.key, false),
        AccountMeta::new_readonly(*accounts.associated_token_program.key, false),
    ];
    metas.extend(remaining_account_metas(remaining_accounts));

    let ix = Instruction {
        program_id: PUMP_PROGRAM_ID,
        accounts: metas,
        data,
    };

    let mut account_infos = vec![
        accounts.payer,
        accounts.mint,
        accounts.bonding_curve,
        accounts.sharing_config,
        accounts.creator_vault,
        accounts.system_program,
        accounts.event_authority,
        accounts.program,
        accounts.creator_vault_quote_token_account,
        accounts.quote_mint,
        accounts.quote_token_program,
        accounts.associated_token_program,
    ];
    account_infos.extend_from_slice(remaining_accounts);

    invoke(&ix, &account_infos)?;

    Ok(())
}

pub fn encode_pump_distribute_creator_fees_v2(initialize_ata: bool) -> Vec<u8> {
    let mut data = PUMP_DISTRIBUTE_CREATOR_FEES_V2_DISCRIMINATOR.to_vec();
    data.push(u8::from(initialize_ata));
    data
}

pub fn remaining_account_metas(accounts: &[AccountInfo<'_>]) -> Vec<AccountMeta> {
    accounts
        .iter()
        .map(|account| {
            if account.is_writable {
                AccountMeta::new(*account.key, account.is_signer)
            } else {
                AccountMeta::new_readonly(*account.key, account.is_signer)
            }
        })
        .collect()
}
