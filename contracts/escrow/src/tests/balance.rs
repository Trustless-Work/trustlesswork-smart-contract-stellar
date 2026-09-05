extern crate std;

use crate::{
    error::ContractError,
    storage::types::{Escrow, Flags, Milestone, Roles, Trustline},
};
use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    vec, Address, Env, IntoVal, Map, String, Symbol,
};

use super::helpers::{create_escrow_contract, create_usdc_token};

#[test]
fn test_get_multiple_escrow_balances_platform_authorized() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);
    let receiver = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let roles = Roles {
        approver: approver.clone(),
        service_provider: service_provider.clone(),
        platform: platform.clone(),
        release_signer: release_signer.clone(),
        dispute_resolver: dispute_resolver.clone(),
        receiver: receiver.clone(),
    };

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Completed"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: false,
        },
    ];

    let escrow_base = Escrow {
        engagement_id: String::from_str(&env, "engagement_registry_1"),
        title: String::from_str(&env, "Escrow for registry test"),
        description: String::from_str(&env, "Test for multiple balances"),
        roles: roles.clone(),
        amount: 50_000_000,
        platform_fee: 100, // 1%
        milestones,
        flags: Flags {
            disputed: false,
            released: false,
            resolved: false,
        },
        trustline: Trustline {
            address: usdc_token.0.address.clone(),
        },
        receiver_memo: 0,
    };

    // Deploy two escrow contracts of the same code and initialize both
    let c1 = create_escrow_contract(&env).client;
    c1.initialize_escrow(&escrow_base);

    let c2 = create_escrow_contract(&env).client;
    c2.initialize_escrow(&escrow_base);

    // Mint funds to both contracts so they have balances
    usdc_token.1.mint(&c1.address, &escrow_base.amount);
    usdc_token.1.mint(&c2.address, &escrow_base.amount);

    // Platform must authorize the query from c1
    // env.mock_all_auths() already mocks auth; we still pass platform as implicit auth signer in SDK
    let res_ok = c1.get_multiple_escrow_balances(&vec![&env, c1.address.clone()]);
    assert_eq!(res_ok.len(), 1);
    assert_eq!(res_ok.get(0).unwrap().address, c1.address);

    // Include any other contract: allowed as long as platform authorizes the call
    let res_two =
        c1.get_multiple_escrow_balances(&vec![&env, c1.address.clone(), c2.address.clone()]);
    assert_eq!(res_two.len(), 2);
}

#[test]
fn test_withdraw_remaining_funds_success() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);
    let trustless_work_address = Address::generate(&env);
    let recipient_a = Address::generate(&env);
    let recipient_b = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let escrow_amount: i128 = 100_000_000;
    let platform_fee: u32 = 300; // 3%

    let roles = Roles {
        approver: approver.clone(),
        service_provider: service_provider.clone(),
        platform: platform.clone(),
        release_signer: release_signer.clone(),
        dispute_resolver: dispute_resolver.clone(),
        receiver: service_provider.clone(),
    };

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Milestone 1"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, ""),
            approved: false,
        },
    ];

    let escrow_properties = Escrow {
        engagement_id: String::from_str(&env, "withdraw_success"),
        title: String::from_str(&env, "Withdraw Success"),
        description: String::from_str(&env, "Test successful fund withdrawal"),
        roles: roles.clone(),
        amount: escrow_amount,
        platform_fee,
        milestones: milestones.clone(),
        flags: Flags {
            disputed: false,
            released: false,
            resolved: false,
        },
        trustline: Trustline {
            address: usdc_token.0.address.clone(),
        },
        receiver_memo: 0,
    };

    let client = create_escrow_contract(&env).client;
    client.initialize_escrow(&escrow_properties);

    // Fund, complete milestone, approve, and release escrow
    usdc_token.1.mint(&approver, &escrow_amount);
    client.fund_escrow(&approver, &escrow_properties, &escrow_amount);
    client.change_milestone_status(
        &0,
        &String::from_str(&env, "Completed"),
        &Some(String::from_str(&env, "Done")),
        &service_provider,
    );
    client.approve_milestone(&0, &approver);
    client.release_funds(&release_signer, &trustless_work_address);

    // Contract balance is now 0. Mint residual/surplus funds directly to simulate overfunding/leftover
    let residual_amount: i128 = 50_000;
    usdc_token.1.mint(&client.address, &residual_amount);
    assert_eq!(usdc_token.0.balance(&client.address), 50_000);

    // Initial balances before withdraw
    let tw_before = usdc_token.0.balance(&trustless_work_address);
    let platform_before = usdc_token.0.balance(&platform);
    let a_before = usdc_token.0.balance(&recipient_a);
    let b_before = usdc_token.0.balance(&recipient_b);

    // Prepare distribution: 20_000 to A, 25_000 to B -> total = 45_000 (leaves 5_000 in contract)
    let mut distributions: Map<Address, i128> = Map::new(&env);
    distributions.set(recipient_a.clone(), 20_000);
    distributions.set(recipient_b.clone(), 25_000);

    let res = client.try_withdraw_remaining_funds(
        &dispute_resolver,
        &trustless_work_address,
        &distributions,
    );
    assert!(res.is_ok(), "Dispute resolver should be able to withdraw remaining funds");

    // Verify that dispute_resolver authorized withdraw_remaining_funds
    assert_eq!(
        env.auths(),
        [(
            dispute_resolver.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    client.address.clone(),
                    Symbol::new(&env, "withdraw_remaining_funds"),
                    (
                        dispute_resolver.clone(),
                        trustless_work_address.clone(),
                        distributions.clone(),
                    )
                        .into_val(&env),
                )),
                sub_invocations: [].into(),
            }
        )]
    );

    // Total distribution = 45,000
    // Fees: TW fee = (45_000 * 30) / 10000 = 135
    // Platform fee = (45_000 * 300) / 10000 = 1350
    // Total fees = 1485
    // Proportional fee deductions:
    // A fee share = (20_000 * 1485) / 45_000 = 660
    // B fee share = (25_000 * 1485) / 45_000 = 825
    let total_dist: i128 = 45_000;
    let tw_fee = (total_dist * 30) / 10000;
    let platform_fee_amount = (total_dist * platform_fee as i128) / 10000;
    let total_fees = tw_fee + platform_fee_amount;

    let fee_share_a = (20_000 * total_fees) / total_dist;
    let fee_share_b = (25_000 * total_fees) / total_dist;

    assert_eq!(usdc_token.0.balance(&trustless_work_address), tw_before + tw_fee);
    assert_eq!(usdc_token.0.balance(&platform), platform_before + platform_fee_amount);
    assert_eq!(usdc_token.0.balance(&recipient_a), a_before + (20_000 - fee_share_a));
    assert_eq!(usdc_token.0.balance(&recipient_b), b_before + (25_000 - fee_share_b));

    // Contract remaining balance
    assert_eq!(usdc_token.0.balance(&client.address), 50_000 - total_dist);
}

#[test]
fn test_withdraw_remaining_funds_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);
    let attacker = Address::generate(&env);
    let trustless_work_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let escrow_properties = Escrow {
        engagement_id: String::from_str(&env, "withdraw_unauth"),
        title: String::from_str(&env, "Withdraw Unauth"),
        description: String::from_str(&env, "Test unauthorized caller rejection"),
        roles: Roles {
            approver: approver.clone(),
            service_provider: service_provider.clone(),
            platform: platform.clone(),
            release_signer: release_signer.clone(),
            dispute_resolver: dispute_resolver.clone(),
            receiver: service_provider.clone(),
        },
        amount: 10_000_000,
        platform_fee: 100,
        milestones: vec![
            &env,
            Milestone {
                description: String::from_str(&env, "M1"),
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
            address: usdc_token.0.address.clone(),
        },
        receiver_memo: 0,
    };

    let client = create_escrow_contract(&env).client;
    client.initialize_escrow(&escrow_properties);

    usdc_token.1.mint(&approver, &10_000_000);
    client.fund_escrow(&approver, &escrow_properties, &10_000_000);
    client.change_milestone_status(
        &0,
        &String::from_str(&env, "Completed"),
        &Some(String::from_str(&env, "Done")),
        &service_provider,
    );
    client.approve_milestone(&0, &approver);
    client.release_funds(&release_signer, &trustless_work_address);

    usdc_token.1.mint(&client.address, &10_000);

    let mut dist: Map<Address, i128> = Map::new(&env);
    dist.set(service_provider.clone(), 10_000);

    // 1. Attacker attempts to withdraw funds (rejected because caller is not dispute_resolver)
    let res = client.try_withdraw_remaining_funds(&attacker, &trustless_work_address, &dist);
    assert_eq!(
        res,
        Err(Ok(ContractError::OnlyDisputeResolverCanExecuteThisFunction))
    );

    // 2. Configured dispute_resolver attempts without matching authorization (require_auth rejects)
    let unauth_res = client
        .mock_auths(&[])
        .try_withdraw_remaining_funds(&dispute_resolver, &trustless_work_address, &dist);
    assert!(
        unauth_res.is_err(),
        "Call without matching dispute_resolver authorization must fail"
    );
}

#[test]
fn test_withdraw_remaining_funds_not_fully_processed() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);
    let trustless_work_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let escrow_properties = Escrow {
        engagement_id: String::from_str(&env, "withdraw_pending"),
        title: String::from_str(&env, "Withdraw Pending"),
        description: String::from_str(&env, "Test calling when escrow is not fully processed"),
        roles: Roles {
            approver: approver.clone(),
            service_provider: service_provider.clone(),
            platform: platform.clone(),
            release_signer: release_signer.clone(),
            dispute_resolver: dispute_resolver.clone(),
            receiver: service_provider.clone(),
        },
        amount: 10_000_000,
        platform_fee: 100,
        milestones: vec![
            &env,
            Milestone {
                description: String::from_str(&env, "M1"),
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
            address: usdc_token.0.address.clone(),
        },
        receiver_memo: 0,
    };

    let client = create_escrow_contract(&env).client;
    client.initialize_escrow(&escrow_properties);

    usdc_token.1.mint(&approver, &10_000_000);
    client.fund_escrow(&approver, &escrow_properties, &10_000_000);

    let mut dist: Map<Address, i128> = Map::new(&env);
    dist.set(service_provider.clone(), 5_000_000);

    // Call while escrow is pending (neither released, resolved, nor disputed)
    let res = client.try_withdraw_remaining_funds(&dispute_resolver, &trustless_work_address, &dist);
    assert_eq!(
        res,
        Err(Ok(ContractError::EscrowNotFullyProcessed))
    );
}

#[test]
fn test_withdraw_remaining_funds_insufficient_balance() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);
    let trustless_work_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let escrow_properties = Escrow {
        engagement_id: String::from_str(&env, "withdraw_insufficient"),
        title: String::from_str(&env, "Withdraw Insufficient"),
        description: String::from_str(&env, "Test exceeding remaining balance"),
        roles: Roles {
            approver: approver.clone(),
            service_provider: service_provider.clone(),
            platform: platform.clone(),
            release_signer: release_signer.clone(),
            dispute_resolver: dispute_resolver.clone(),
            receiver: service_provider.clone(),
        },
        amount: 10_000_000,
        platform_fee: 100,
        milestones: vec![
            &env,
            Milestone {
                description: String::from_str(&env, "M1"),
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
            address: usdc_token.0.address.clone(),
        },
        receiver_memo: 0,
    };

    let client = create_escrow_contract(&env).client;
    client.initialize_escrow(&escrow_properties);

    usdc_token.1.mint(&approver, &10_000_000);
    client.fund_escrow(&approver, &escrow_properties, &10_000_000);
    client.change_milestone_status(
        &0,
        &String::from_str(&env, "Completed"),
        &Some(String::from_str(&env, "Done")),
        &service_provider,
    );
    client.approve_milestone(&0, &approver);
    client.release_funds(&release_signer, &trustless_work_address);

    // Contract has 10_000 residual balance
    usdc_token.1.mint(&client.address, &10_000);

    // Request 20_000 (exceeds balance of 10_000)
    let mut dist: Map<Address, i128> = Map::new(&env);
    dist.set(service_provider.clone(), 20_000);

    let res = client.try_withdraw_remaining_funds(&dispute_resolver, &trustless_work_address, &dist);
    assert_eq!(
        res,
        Err(Ok(ContractError::InsufficientFundsForResolution))
    );
}

#[test]
fn test_withdraw_remaining_funds_after_dispute_resolved() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);
    let trustless_work_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let escrow_amount: i128 = 50_000_000;
    let platform_fee: u32 = 200; // 2%

    let escrow_properties = Escrow {
        engagement_id: String::from_str(&env, "withdraw_after_dispute"),
        title: String::from_str(&env, "Withdraw After Dispute"),
        description: String::from_str(&env, "Test sweeping funds after dispute resolution"),
        roles: Roles {
            approver: approver.clone(),
            service_provider: service_provider.clone(),
            platform: platform.clone(),
            release_signer: release_signer.clone(),
            dispute_resolver: dispute_resolver.clone(),
            receiver: service_provider.clone(),
        },
        amount: escrow_amount,
        platform_fee,
        milestones: vec![
            &env,
            Milestone {
                description: String::from_str(&env, "M1"),
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
            address: usdc_token.0.address.clone(),
        },
        receiver_memo: 0,
    };

    let client = create_escrow_contract(&env).client;
    client.initialize_escrow(&escrow_properties);

    usdc_token.1.mint(&approver, &escrow_amount);
    client.fund_escrow(&approver, &escrow_properties, &escrow_amount);

    // Escrow enters dispute
    client.dispute_escrow(&approver);

    // Dispute resolved (split remaining balance: 25M approver, 25M provider)
    let mut resolution_dist: Map<Address, i128> = Map::new(&env);
    resolution_dist.set(approver.clone(), 25_000_000);
    resolution_dist.set(service_provider.clone(), 25_000_000);
    client.resolve_dispute(&dispute_resolver, &trustless_work_address, &resolution_dist);

    // Contract balance is now 0. Subsequent residual funds arrive (e.g. overpayment or refund)
    let residual: i128 = 30_000;
    usdc_token.1.mint(&client.address, &residual);

    // Sweeping residual funds is allowed because escrow.flags.resolved == true
    let mut sweep_dist: Map<Address, i128> = Map::new(&env);
    sweep_dist.set(approver.clone(), 30_000);

    let res = client.try_withdraw_remaining_funds(&dispute_resolver, &trustless_work_address, &sweep_dist);
    assert!(res.is_ok(), "Dispute resolver should be able to sweep remaining funds after dispute resolution");
    assert_eq!(usdc_token.0.balance(&client.address), 0);
}

