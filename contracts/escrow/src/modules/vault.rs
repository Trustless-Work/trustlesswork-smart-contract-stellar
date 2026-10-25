use soroban_sdk::auth::{ContractContext, InvokerContractAuthEntry, SubContractInvocation};
use soroban_sdk::{contractclient, vec, Address, Env, IntoVal, Symbol, Val, Vec};

use crate::error::ContractError;
use crate::storage::types::{AssetStrategySet, Trustline};

/// The subset of the DeFindex vault interface the escrow uses.
#[allow(dead_code)]
#[contractclient(name = "VaultClient")]
pub trait VaultInterface {
    fn deposit(
        e: Env,
        amounts_desired: Vec<i128>,
        amounts_min: Vec<i128>,
        from: Address,
        invest: bool,
    ) -> Val;
    fn withdraw(
        e: Env,
        withdraw_shares: i128,
        min_amounts_out: Vec<i128>,
        from: Address,
    ) -> Vec<i128>;
    fn balance(e: Env, id: Address) -> i128;
    fn get_asset_amounts_per_shares(e: Env, vault_shares: i128) -> Vec<i128>;
    fn get_assets(e: Env) -> Vec<AssetStrategySet>;
}

pub struct VaultManager;

impl VaultManager {
    /// The vault must manage exactly one asset: the escrow's trustline.
    pub fn validate_asset(
        e: &Env,
        vault: &Address,
        trustline: &Trustline,
    ) -> Result<(), ContractError> {
        let assets = VaultClient::new(e, vault).get_assets();
        if assets.len() != 1 || assets.get(0).unwrap().address != trustline.address {
            return Err(ContractError::VaultAssetMismatch);
        }
        Ok(())
    }

    pub fn deposit(e: &Env, vault: &Address, token: &Address, amount: i128) {
        let escrow = e.current_contract_address();
        // The vault pulls the tokens with a nested `transfer`, which needs the escrow's auth.
        e.authorize_as_current_contract(vec![
            e,
            InvokerContractAuthEntry::Contract(SubContractInvocation {
                context: ContractContext {
                    contract: token.clone(),
                    fn_name: Symbol::new(e, "transfer"),
                    args: (escrow.clone(), vault.clone(), amount).into_val(e),
                },
                sub_invocations: vec![e],
            }),
        ]);
        VaultClient::new(e, vault).deposit(&vec![e, amount], &vec![e, amount], &escrow, &true);
    }

    /// Burns all the escrow's shares, returning the underlying to the escrow.
    pub fn redeem_all(e: &Env, vault: &Address) {
        let escrow = e.current_contract_address();
        let client = VaultClient::new(e, vault);
        let shares = client.balance(&escrow);
        if shares > 0 {
            client.withdraw(&shares, &vec![e, 0], &escrow);
        }
    }

    /// Returns `(shares, underlying)` held by `holder` in the vault.
    pub fn position(e: &Env, vault: &Address, holder: &Address) -> (i128, i128) {
        let client = VaultClient::new(e, vault);
        let shares = client.balance(holder);
        if shares <= 0 {
            return (0, 0);
        }
        let underlying = client
            .get_asset_amounts_per_shares(&shares)
            .get(0)
            .unwrap_or(0);
        (shares, underlying)
    }
}
