extern crate std;

use crate::storage::types::{Escrow, Flags, Milestone, Roles, Trustline};
use soroban_sdk::{testutils::Address as _, vec, Address, Env, String};

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
