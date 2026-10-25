extern crate std;

use crate::contract::EscrowContractClient;
use crate::error::ContractError;
use crate::storage::types::{AssetStrategySet, Escrow, Flags, Milestone, Roles, Trustline};
use soroban_sdk::testutils::{Address as _, MockAuth, MockAuthInvoke};
use soroban_sdk::token::{Client as TokenClient, StellarAssetClient as TokenAdminClient};
use soroban_sdk::{
    contract, contractimpl, contracttype, vec, Address, Env, IntoVal, Map, String, Vec,
};

use super::helpers::{create_escrow_contract, create_usdc_token};

/// Share-based vault with DeFindex's interface. Yield is simulated by minting
/// the asset straight to the vault, which raises the value of each share.
#[contract]
pub struct MockVault;

#[contracttype]
enum MockKey {
    Asset,
    TotalShares,
    Shares(Address),
}

fn get_i128(e: &Env, key: &MockKey) -> i128 {
    e.storage().instance().get(key).unwrap_or(0)
}

fn asset_client(e: &Env) -> TokenClient<'_> {
    let asset: Address = e.storage().instance().get(&MockKey::Asset).unwrap();
    TokenClient::new(e, &asset)
}

#[contractimpl]
impl MockVault {
    pub fn __constructor(e: Env, asset: Address) {
        e.storage().instance().set(&MockKey::Asset, &asset);
    }

    pub fn deposit(
        e: Env,
        amounts_desired: Vec<i128>,
        _amounts_min: Vec<i128>,
        from: Address,
        _invest: bool,
    ) -> (Vec<i128>, i128) {
        from.require_auth();
        let amount = amounts_desired.get(0).unwrap();
        let token = asset_client(&e);
        let vault = e.current_contract_address();
        let total_assets = token.balance(&vault);
        let total_shares = get_i128(&e, &MockKey::TotalShares);
        let shares = if total_shares == 0 {
            amount
        } else {
            amount * total_shares / total_assets
        };
        token.transfer(&from, &vault, &amount);
        let key = MockKey::Shares(from);
        let held = get_i128(&e, &key);
        e.storage().instance().set(&key, &(held + shares));
        e.storage()
            .instance()
            .set(&MockKey::TotalShares, &(total_shares + shares));
        (vec![&e, amount], shares)
    }

    pub fn withdraw(
        e: Env,
        withdraw_shares: i128,
        _min_amounts_out: Vec<i128>,
        from: Address,
    ) -> Vec<i128> {
        from.require_auth();
        let out = Self::get_asset_amounts_per_shares(e.clone(), withdraw_shares)
            .get(0)
            .unwrap();
        let key = MockKey::Shares(from.clone());
        let held = get_i128(&e, &key);
        assert!(held >= withdraw_shares);
        e.storage().instance().set(&key, &(held - withdraw_shares));
        let total_shares = get_i128(&e, &MockKey::TotalShares);
        e.storage()
            .instance()
            .set(&MockKey::TotalShares, &(total_shares - withdraw_shares));
        asset_client(&e).transfer(&e.current_contract_address(), &from, &out);
        vec![&e, out]
    }

    pub fn balance(e: Env, id: Address) -> i128 {
        get_i128(&e, &MockKey::Shares(id))
    }

    pub fn get_asset_amounts_per_shares(e: Env, vault_shares: i128) -> Vec<i128> {
        let total_shares = get_i128(&e, &MockKey::TotalShares);
        if total_shares == 0 {
            return vec![&e, 0];
        }
        let total_assets = asset_client(&e).balance(&e.current_contract_address());
        vec![&e, vault_shares * total_assets / total_shares]
    }

    pub fn get_assets(e: Env) -> Vec<AssetStrategySet> {
        let asset: Address = e.storage().instance().get(&MockKey::Asset).unwrap();
        vec![
            &e,
            AssetStrategySet {
                address: asset,
                strategies: vec![&e],
            },
        ]
    }
}

struct Setup<'a> {
    env: Env,
    escrow: EscrowContractClient<'a>,
    token: TokenClient<'a>,
    token_admin: TokenAdminClient<'a>,
    vault: Address,
    properties: Escrow,
    approver: Address,
    service_provider: Address,
    release_signer: Address,
    dispute_resolver: Address,
    receiver: Address,
    platform: Address,
    funder: Address,
    trustless_work: Address,
}

const AMOUNT: i128 = 100_000_000;
const PLATFORM_FEE_BPS: u32 = 500;

fn setup<'a>() -> Setup<'a> {
    let env = Env::default();
    env.mock_all_auths();

    let token_issuer = Address::generate(&env);
    let (token, token_admin) = create_usdc_token(&env, &token_issuer);
    let vault = env.register(MockVault, (token.address.clone(),));

    let escrow = create_escrow_contract(&env).client;

    let approver = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);
    let receiver = Address::generate(&env);
    let platform = Address::generate(&env);
    let funder = Address::generate(&env);
    let trustless_work = Address::generate(&env);

    let properties = Escrow {
        engagement_id: String::from_str(&env, "yield"),
        title: String::from_str(&env, "Yield escrow"),
        description: String::from_str(&env, "Funds earn yield in a vault"),
        roles: Roles {
            approver: approver.clone(),
            service_provider: service_provider.clone(),
            platform: platform.clone(),
            release_signer: release_signer.clone(),
            dispute_resolver: dispute_resolver.clone(),
            receiver: receiver.clone(),
        },
        amount: AMOUNT,
        platform_fee: PLATFORM_FEE_BPS,
        milestones: vec![
            &env,
            Milestone {
                description: String::from_str(&env, "Deliverable"),
                status: String::from_str(&env, "Pending"),
                evidence: String::from_str(&env, ""),
                approved: false,
            },
        ],
        flags: Flags {
            disputed: false,
            released: false,
            resolved: false,
        },
        trustline: Trustline {
            address: token.address.clone(),
        },
        receiver_memo: 0,
        vault: Some(vault.clone()),
    };

    token_admin.mint(&funder, &(AMOUNT * 2));

    Setup {
        env,
        escrow,
        token,
        token_admin,
        vault,
        properties,
        approver,
        service_provider,
        release_signer,
        dispute_resolver,
        receiver,
        platform,
        funder,
        trustless_work,
    }
}

fn init_and_fund(s: &Setup) -> Escrow {
    let escrow = s.escrow.initialize_escrow(&s.properties);
    s.escrow.fund_escrow(&s.funder, &escrow, &AMOUNT);
    escrow
}

fn add_yield(s: &Setup, amount: i128) {
    s.token_admin.mint(&s.vault, &amount);
}

/// TW takes 0.3%, the platform PLATFORM_FEE_BPS, both from `gross`.
fn fees(gross: i128) -> (i128, i128) {
    (
        gross * 30 / 10_000,
        gross * PLATFORM_FEE_BPS as i128 / 10_000,
    )
}

#[test]
fn test_initialize_rejects_vault_with_other_asset() {
    let s = setup();
    let (other_token, _) = create_usdc_token(&s.env, &Address::generate(&s.env));
    let other_vault = s.env.register(MockVault, (other_token.address.clone(),));

    let mut properties = s.properties.clone();
    properties.vault = Some(other_vault);

    let result = s.escrow.try_initialize_escrow(&properties);
    assert_eq!(result.err(), Some(Ok(ContractError::VaultAssetMismatch)));
}

#[test]
fn test_fund_deposits_into_vault() {
    let s = setup();
    init_and_fund(&s);

    assert_eq!(s.token.balance(&s.escrow.address), 0);
    assert_eq!(s.token.balance(&s.vault), AMOUNT);

    let position = s.escrow.get_vault_position().unwrap();
    assert_eq!(position.vault, s.vault);
    assert_eq!(position.underlying, AMOUNT);

    let balances = s
        .escrow
        .get_multiple_escrow_balances(&vec![&s.env, s.escrow.address.clone()]);
    assert_eq!(balances.get(0).unwrap().balance, AMOUNT);
}

#[test]
fn test_balance_includes_yield() {
    let s = setup();
    init_and_fund(&s);
    add_yield(&s, 5_000_000);

    let balances = s
        .escrow
        .get_multiple_escrow_balances(&vec![&s.env, s.escrow.address.clone()]);
    assert_eq!(balances.get(0).unwrap().balance, AMOUNT + 5_000_000);
}

#[test]
fn test_release_pays_principal_and_yield_to_receiver() {
    let s = setup();
    init_and_fund(&s);
    let earned = 10_000_000;
    add_yield(&s, earned);

    s.escrow.approve_milestone(&0, &s.approver);
    s.escrow.release_funds(&s.release_signer, &s.trustless_work);

    let gross = AMOUNT + earned;
    let (tw_fee, platform_fee) = fees(gross);
    assert_eq!(s.token.balance(&s.trustless_work), tw_fee);
    assert_eq!(s.token.balance(&s.platform), platform_fee);
    assert_eq!(s.token.balance(&s.receiver), gross - tw_fee - platform_fee);
    assert_eq!(s.token.balance(&s.escrow.address), 0);
    assert_eq!(s.escrow.get_vault_position().unwrap().shares, 0);
}

#[test]
fn test_release_requires_full_funding() {
    let s = setup();
    let escrow = s.escrow.initialize_escrow(&s.properties);
    s.escrow.fund_escrow(&s.funder, &escrow, &(AMOUNT / 2));
    // Yield alone never makes up for missing funding.
    add_yield(&s, AMOUNT);

    s.escrow.approve_milestone(&0, &s.approver);
    let result = s
        .escrow
        .try_release_funds(&s.release_signer, &s.trustless_work);
    assert_eq!(
        result,
        Err(Ok(ContractError::EscrowBalanceNotEnoughToSendEarnings))
    );
}

#[test]
fn test_resolve_dispute_allows_partial_then_withdraw_sweeps_rest() {
    let s = setup();
    init_and_fund(&s);
    let earned = 4_000_000;
    add_yield(&s, earned);

    s.escrow.dispute_escrow(&s.service_provider);

    // The resolver splits the principal; the yield stays behind.
    let mut distributions: Map<Address, i128> = Map::new(&s.env);
    distributions.set(s.receiver.clone(), 60_000_000);
    distributions.set(s.funder.clone(), 40_000_000);
    s.escrow
        .resolve_dispute(&s.dispute_resolver, &s.trustless_work, &distributions);

    assert_eq!(s.token.balance(&s.escrow.address), earned);
    assert_eq!(s.escrow.get_vault_position().unwrap().shares, 0);

    let mut rest: Map<Address, i128> = Map::new(&s.env);
    rest.set(s.funder.clone(), earned);
    s.escrow
        .withdraw_remaining_funds(&s.dispute_resolver, &s.trustless_work, &rest);
    assert_eq!(s.token.balance(&s.escrow.address), 0);
}

#[test]
fn test_resolve_dispute_rejects_more_than_balance() {
    let s = setup();
    init_and_fund(&s);
    s.escrow.dispute_escrow(&s.service_provider);

    let mut distributions: Map<Address, i128> = Map::new(&s.env);
    distributions.set(s.receiver.clone(), AMOUNT + 1);
    let result =
        s.escrow
            .try_resolve_dispute(&s.dispute_resolver, &s.trustless_work, &distributions);
    assert_eq!(
        result,
        Err(Ok(ContractError::InsufficientFundsForResolution))
    );
}

#[test]
fn test_fund_after_release_stays_idle() {
    let s = setup();
    let escrow = init_and_fund(&s);
    s.escrow.approve_milestone(&0, &s.approver);
    s.escrow.release_funds(&s.release_signer, &s.trustless_work);

    let released = s.escrow.get_escrow();
    s.escrow.fund_escrow(&s.funder, &released, &1_000);
    assert_eq!(s.token.balance(&s.escrow.address), 1_000);
    assert_eq!(s.escrow.get_vault_position().unwrap().shares, 0);
    let _ = escrow;
}

#[test]
fn test_vault_cannot_change_after_funding() {
    let s = setup();
    let escrow = init_and_fund(&s);
    let other_vault = s.env.register(MockVault, (s.token.address.clone(),));

    let mut updated = escrow.clone();
    updated.vault = Some(other_vault);
    let result = s.escrow.try_update_escrow(&s.platform, &updated);
    assert_eq!(
        result.err(),
        Some(Ok(ContractError::EscrowPropertiesMismatch))
    );
}

#[test]
fn test_fund_authorizes_vault_pull_without_mocked_auth() {
    let s = setup();
    let escrow = s.escrow.initialize_escrow(&s.properties);

    // Only the funder signs; the vault's pull from the escrow must be
    // authorized by the escrow itself.
    s.env.mock_auths(&[MockAuth {
        address: &s.funder,
        invoke: &MockAuthInvoke {
            contract: &s.escrow.address,
            fn_name: "fund_escrow",
            args: (s.funder.clone(), escrow.clone(), AMOUNT).into_val(&s.env),
            sub_invokes: &[MockAuthInvoke {
                contract: &s.token.address,
                fn_name: "transfer",
                args: (s.funder.clone(), s.escrow.address.clone(), AMOUNT).into_val(&s.env),
                sub_invokes: &[],
            }],
        },
    }]);
    s.escrow.fund_escrow(&s.funder, &escrow, &AMOUNT);

    assert_eq!(s.token.balance(&s.vault), AMOUNT);
}
