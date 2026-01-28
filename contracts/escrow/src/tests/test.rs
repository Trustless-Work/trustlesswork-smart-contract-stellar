#![cfg(test)]

extern crate std;

use crate::contract::EscrowContract;
use crate::contract::EscrowContractClient;
use crate::storage::types::{Escrow, Flags, Milestone, MilestoneUpdate, Roles, Trustline};

use soroban_sdk::{testutils::Address as _, token, vec, Address, Env, Map, String};
use token::Client as TokenClient;
use token::StellarAssetClient as TokenAdminClient;
// use test_token::token::{Token, TokenClient};

fn create_usdc_token<'a>(e: &Env, admin: &Address) -> (TokenClient<'a>, TokenAdminClient<'a>) {
    let sac = e.register_stellar_asset_contract_v2(admin.clone());
    (
        TokenClient::new(e, &sac.address()),
        TokenAdminClient::new(e, &sac.address()),
    )
}

struct TestData<'a> {
    client: EscrowContractClient<'a>,
}

fn create_escrow_contract<'a>(env: &Env) -> TestData<'a> {
    env.mock_all_auths();
    let client = EscrowContractClient::new(env, &env.register(EscrowContract {}, ()));

    TestData { client }
}

#[test]
fn test_initialize_excrow() {
    let env = Env::default();
    env.mock_all_auths();

    let approver_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let amount: i128 = 100_000_000;
    let service_provider_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let _receiver_address = Address::generate(&env);
    let platform_fee = 3 * 100;
    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: false,
        },
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: false,
        },
    ];

    let usdc_token = create_usdc_token(&env, &admin);

    let engagement_id = String::from_str(&env, "41431");

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        receiver: service_provider_address.clone(),
        observers: vec![&env],
    };

    let flags: Flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles,
        amount: amount,
        platform_fee: platform_fee,
        milestones: milestones,
        flags,
        trustline,
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_approver = test_data.client;

    let initialized_escrow = escrow_approver.initialize_escrow(&escrow_properties);

    let escrow = escrow_approver.get_escrow();
    assert_eq!(escrow.engagement_id, initialized_escrow.engagement_id);
    assert_eq!(escrow.roles.approver, escrow_properties.roles.approver);
    assert_eq!(
        escrow.roles.service_provider,
        escrow_properties.roles.service_provider
    );
    assert_eq!(
        escrow.roles.platform_address,
        escrow_properties.roles.platform_address
    );
    assert_eq!(escrow.amount, amount);
    assert_eq!(escrow.platform_fee, platform_fee);
    assert_eq!(escrow.milestones, escrow_properties.milestones);
    assert_eq!(
        escrow.roles.release_signer,
        escrow_properties.roles.release_signer
    );
    assert_eq!(
        escrow.roles.dispute_resolver,
        escrow_properties.roles.dispute_resolver
    );
    assert_eq!(escrow.roles.receiver, escrow_properties.roles.receiver);
    assert_eq!(escrow.receiver_memo, escrow_properties.receiver_memo);

    let result = escrow_approver.try_initialize_escrow(&escrow_properties);
    assert!(result.is_err());
}

#[test]
fn test_update_escrow() {
    let env = Env::default();
    env.mock_all_auths();

    let approver_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let _receiver_address = Address::generate(&env);

    let amount: i128 = 100_000_000;
    let platform_fee = 3 * 100;

    let initial_milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: false,
        },
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: false,
        },
    ];

    let usdc_token = create_usdc_token(&env, &admin);

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        receiver: service_provider_address.clone(),
        observers: vec![&env],
    };

    let flags: Flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let engagement_id = String::from_str(&env, "test_escrow_2");
    let initial_escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles: roles.clone(),
        amount: amount,
        platform_fee: platform_fee,
        milestones: initial_milestones.clone(),
        flags: flags.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&initial_escrow_properties);

    // Create a new updated escrow properties
    let new_milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone updated"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: false,
        },
        Milestone {
            description: String::from_str(&env, "Second milestone updated"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: false,
        },
        Milestone {
            description: String::from_str(&env, "Third milestone new"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: false,
        },
    ];

    let updated_escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow Updated"),
        description: String::from_str(&env, "Test Escrow Description Updated"),
        roles,
        amount: amount * 2,
        platform_fee: platform_fee * 2,
        milestones: new_milestones.clone(),
        flags,
        trustline,
        receiver_memo: 0,
    };

    // Update escrow properties
    let _updated_escrow =
        escrow_approver.update_escrow(&platform_address, &updated_escrow_properties);

    // Verify updated escrow properties
    let escrow = escrow_approver.get_escrow();
    assert_eq!(escrow.title, updated_escrow_properties.title);
    assert_eq!(escrow.description, updated_escrow_properties.description);
    assert_eq!(escrow.amount, updated_escrow_properties.amount);
    assert_eq!(escrow.platform_fee, updated_escrow_properties.platform_fee);
    assert_eq!(escrow.milestones, updated_escrow_properties.milestones);
    assert_eq!(
        escrow.roles.release_signer,
        updated_escrow_properties.roles.release_signer
    );
    assert_eq!(
        escrow.roles.dispute_resolver,
        updated_escrow_properties.roles.dispute_resolver
    );
    assert_eq!(
        escrow.roles.receiver,
        updated_escrow_properties.roles.receiver
    );
    assert_eq!(
        escrow.receiver_memo,
        updated_escrow_properties.receiver_memo
    );

    // Try to update escrow properties without platform address (should fail)
    let non_platform_address = Address::generate(&env);
    let result =
        escrow_approver.try_update_escrow(&non_platform_address, &updated_escrow_properties);
    assert!(result.is_err());
}

#[test]
fn test_update_escrow_platform_fee_too_high() {
    let env = Env::default();
    env.mock_all_auths();

    let approver_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);

    let amount: i128 = 10_000_000;
    let platform_fee_valid = 50 * 100; // 50%
    let platform_fee_invalid = 100 * 100; // 100% (should fail because cap is 99%)

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "M1"),
            status: String::from_str(&env, "pending"),
            evidence: String::from_str(&env, "e"),
            approved: false,
        },
    ];

    let (token_client, _admin_client) = create_usdc_token(&env, &admin);
    let trustline: Trustline = Trustline { address: token_client.address.clone() };

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        receiver: service_provider_address.clone(),
        observers: vec![&env],
    };

    let flags: Flags = Flags { disputed: false, released: false, resolved: false };

    let initial_escrow: Escrow = Escrow {
        engagement_id: String::from_str(&env, "pf_valid"),
        title: String::from_str(&env, "Escrow"),
        description: String::from_str(&env, "Desc"),
        roles: roles.clone(),
        amount,
        platform_fee: platform_fee_valid,
        milestones: milestones.clone(),
        flags: flags.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let client = test_data.client;
    client.initialize_escrow(&initial_escrow);

    // Attempt invalid update (no funds path so full modification allowed but platform_fee cap enforced)
    let invalid_update: Escrow = Escrow {
        engagement_id: String::from_str(&env, "pf_valid"),
        title: String::from_str(&env, "Escrow"),
        description: String::from_str(&env, "Desc"),
        roles: roles.clone(),
        amount,
        platform_fee: platform_fee_invalid,
        milestones: milestones.clone(),
        flags: flags.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let res = client.try_update_escrow(&platform_address, &invalid_update);
    assert!(res.is_err(), "Update should fail with platform fee > 99% cap");
}

#[test]
fn test_initialize_escrow_platform_fee_too_high() {
    let env = Env::default();
    env.mock_all_auths();

    let approver_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);

    let amount: i128 = 10_000_000;
    let platform_fee_invalid = 100 * 100; // 100%

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "M1"),
            status: String::from_str(&env, "pending"),
            evidence: String::from_str(&env, "e"),
            approved: false,
        },
    ];

    let (token_client, _admin_client) = create_usdc_token(&env, &admin);
    let trustline: Trustline = Trustline { address: token_client.address.clone() };

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        receiver: service_provider_address.clone(),
        observers: vec![&env],
    };

    let flags: Flags = Flags { disputed: false, released: false, resolved: false };

    let invalid_escrow: Escrow = Escrow {
        engagement_id: String::from_str(&env, "pf_invalid_init"),
        title: String::from_str(&env, "Escrow"),
        description: String::from_str(&env, "Desc"),
        roles,
        amount,
        platform_fee: platform_fee_invalid,
        milestones: milestones.clone(),
        flags,
        trustline,
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let client = test_data.client;
    let res = client.try_initialize_escrow(&invalid_escrow);
    assert!(res.is_err(), "Initialization should fail with platform fee > 99% cap");
}

#[test]
fn test_append_milestones_with_funds() {
    let env = Env::default();
    env.mock_all_auths();

    let approver_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let receiver_address = service_provider_address.clone();

    let amount: i128 = 100_000_000;
    let platform_fee = 3 * 100;

    let (token_client, token_admin) = create_usdc_token(&env, &admin);

    let initial_milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: false,
        },
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: false,
        },
    ];

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        receiver: receiver_address.clone(),
        observers: vec![&env],
    };

    let flags: Flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
    };

    let trustline: Trustline = Trustline {
        address: token_client.address.clone(),
    };

    let engagement_id = String::from_str(&env, "append_with_funds");
    let initial_escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles: roles.clone(),
        amount: amount,
        platform_fee: platform_fee,
        milestones: initial_milestones.clone(),
        flags: flags.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&initial_escrow_properties);

    // Fund the escrow contract
    token_admin.mint(&approver_address, &amount);
    escrow_approver.fund_escrow(&approver_address, &initial_escrow_properties, &amount);

    // Build updated properties with milestones appended, all other fields identical
    let updated_milestones = vec![
        &env,
        initial_escrow_properties.milestones.get(0).unwrap(),
        initial_escrow_properties.milestones.get(1).unwrap(),
        Milestone {
            description: String::from_str(&env, "Third milestone new"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: false,
        },
    ];

    let updated_escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles: roles.clone(),
        amount: amount,
        platform_fee: platform_fee,
        milestones: updated_milestones.clone(),
        flags: flags.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    escrow_approver.update_escrow(&platform_address, &updated_escrow_properties);

    let escrow = escrow_approver.get_escrow();
    assert_eq!(escrow.milestones.len(), 3);
    assert_eq!(escrow.milestones.get(0).unwrap(), initial_escrow_properties.milestones.get(0).unwrap());
    assert_eq!(escrow.milestones.get(1).unwrap(), initial_escrow_properties.milestones.get(1).unwrap());
    // Ensure non-milestone properties unchanged
    assert_eq!(escrow.engagement_id, initial_escrow_properties.engagement_id);
    assert_eq!(escrow.title, initial_escrow_properties.title);
    assert_eq!(escrow.description, initial_escrow_properties.description);
    assert!(escrow.roles == initial_escrow_properties.roles);
    assert_eq!(escrow.amount, initial_escrow_properties.amount);
    assert_eq!(escrow.platform_fee, initial_escrow_properties.platform_fee);
    assert!(escrow.flags == initial_escrow_properties.flags);
    assert!(escrow.trustline == initial_escrow_properties.trustline);
    assert_eq!(escrow.receiver_memo, initial_escrow_properties.receiver_memo);
}

#[test]
fn test_append_milestones_with_funds_and_existing_approved() {
    // This test validates that after approving an existing milestone, the contract still allows
    // appending new milestones (while keeping existing ones unchanged) when the escrow has funds.
    let env = Env::default();
    env.mock_all_auths();

    let approver_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let receiver_address = service_provider_address.clone();

    let amount: i128 = 50_000_000;
    let platform_fee = 3 * 100;

    let (token_client, token_admin) = create_usdc_token(&env, &admin);

    let initial_milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: false,
        },
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: false,
        },
    ];

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        receiver: receiver_address.clone(),
        observers: vec![&env],
    };

    let flags: Flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
    };

    let trustline: Trustline = Trustline {
        address: token_client.address.clone(),
    };

    let engagement_id = String::from_str(&env, "append_with_funds_and_approved");
    let initial_escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow Approved"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles: roles.clone(),
        amount: amount,
        platform_fee: platform_fee,
        milestones: initial_milestones.clone(),
        flags: flags.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_client = test_data.client;
    escrow_client.initialize_escrow(&initial_escrow_properties);

    // Fund the escrow contract
    token_admin.mint(&approver_address, &amount);
    escrow_client.fund_escrow(&approver_address, &initial_escrow_properties, &amount);

    // Approve the first milestone
    let milestone_indices = vec![&env, 0];
    escrow_client.approve_milestones(&milestone_indices, &approver_address);
    let after_approval = escrow_client.get_escrow();
    assert!(after_approval.milestones.get(0).unwrap().approved);

    // Build updated properties with a new milestone appended (unapproved)
    let updated_milestones = vec![
        &env,
        after_approval.milestones.get(0).unwrap(),
        after_approval.milestones.get(1).unwrap(),
        Milestone {
            description: String::from_str(&env, "Third milestone new"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: false,
        },
    ];

    let updated_escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow Approved"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles: roles.clone(),
        amount: amount,
        platform_fee: platform_fee,
        milestones: updated_milestones.clone(),
        flags: flags.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    escrow_client.update_escrow(&platform_address, &updated_escrow_properties);
    let final_escrow = escrow_client.get_escrow();

    assert_eq!(final_escrow.milestones.len(), 3);
    assert!(final_escrow.milestones.get(0).unwrap().approved, "Existing approved milestone should remain approved");
    assert_eq!(final_escrow.milestones.get(1).unwrap(), after_approval.milestones.get(1).unwrap());
    assert!(!final_escrow.milestones.get(2).unwrap().approved, "Appended milestone should start unapproved");
    // Ensure other properties unchanged
    assert_eq!(final_escrow.engagement_id, initial_escrow_properties.engagement_id);
    assert_eq!(final_escrow.title, initial_escrow_properties.title);
    assert_eq!(final_escrow.description, initial_escrow_properties.description);
    assert!(final_escrow.roles == initial_escrow_properties.roles);
    assert_eq!(final_escrow.amount, initial_escrow_properties.amount);
    assert_eq!(final_escrow.platform_fee, initial_escrow_properties.platform_fee);
    assert!(final_escrow.flags == initial_escrow_properties.flags);
    assert!(final_escrow.trustline == initial_escrow_properties.trustline);
    assert_eq!(final_escrow.receiver_memo, initial_escrow_properties.receiver_memo);
}

#[test]
fn test_change_milestone_status_and_approved() {
    let env = Env::default();
    env.mock_all_auths();

    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let usdc_token = create_usdc_token(&env, &admin);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let amount: i128 = 100_000_000;
    let platform_fee = 3 * 100;

    let initial_milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Milestone 1"),
            status: String::from_str(&env, "in-progress"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: false,
        },
        Milestone {
            description: String::from_str(&env, "Milestone 2"),
            status: String::from_str(&env, "in-progress"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: false,
        },
    ];

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        receiver: service_provider_address.clone(),
        observers: vec![&env],
    };

    let flags: Flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let engagement_id = String::from_str(&env, "test_escrow");
    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles: roles.clone(),
        amount: amount,
        platform_fee: platform_fee,
        milestones: initial_milestones.clone(),
        flags: flags.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&escrow_properties);

    // Change milestone status (valid case)
    let milestone_updates = vec![
        &env,
        MilestoneUpdate {
            index: 0,
            status: String::from_str(&env, "completed"),
            evidence: Some(String::from_str(&env, "New evidence")),
        },
    ];
    escrow_approver.change_milestone_status(
        &milestone_updates,
        &service_provider_address,
    );

    let updated_escrow = escrow_approver.get_escrow();
    assert_eq!(updated_escrow.milestones.get(0).unwrap().status, String::from_str(&env, "completed"));
    assert_eq!(
        updated_escrow.milestones.get(0).unwrap().evidence,
        String::from_str(&env, "New evidence")
    );

    // Change milestone approved (valid case)
    let milestone_indices = vec![&env, 0 as i128];
    escrow_approver.approve_milestones(&milestone_indices, &approver_address);

    let final_escrow = escrow_approver.get_escrow();
    assert!(final_escrow.milestones.get(0).unwrap().approved);

    let invalid_index = 10 as i128;

    let invalid_updates = vec![
        &env,
        MilestoneUpdate {
            index: invalid_index,
            status: String::from_str(&env, "completed"),
            evidence: Some(String::from_str(&env, "New evidence")),
        },
    ];
    let result = escrow_approver.try_change_milestone_status(
        &invalid_updates,
        &service_provider_address,
    );
    assert!(result.is_err());

    let invalid_indices = vec![&env, invalid_index];
    let result = escrow_approver.try_approve_milestones(&invalid_indices, &approver_address);
    assert!(result.is_err());

    let unauthorized_address = Address::generate(&env);

    // Test for `change_status` by invalid service provider
    let valid_updates = vec![
        &env,
        MilestoneUpdate {
            index: 0,
            status: String::from_str(&env, "completed"),
            evidence: Some(String::from_str(&env, "New evidence")),
        },
    ];
    let result = escrow_approver.try_change_milestone_status(
        &valid_updates,
        &unauthorized_address,
    );
    assert!(result.is_err());

    // Test for `change_approved` by invalid approver
    let valid_indices = vec![&env, 0 as i128];
    let result = escrow_approver.try_approve_milestones(&valid_indices, &unauthorized_address);
    assert!(result.is_err());

    // Test changing multiple milestones at once
    let multiple_updates = vec![
        &env,
        MilestoneUpdate {
            index: 0,
            status: String::from_str(&env, "reviewed"),
            evidence: Some(String::from_str(&env, "Batch update evidence")),
        },
        MilestoneUpdate {
            index: 1,
            status: String::from_str(&env, "reviewed"),
            evidence: Some(String::from_str(&env, "Batch update evidence")),
        },
    ];
    
    escrow_approver.change_milestone_status(
        &multiple_updates,
        &service_provider_address,
    );

    let batch_updated_escrow = escrow_approver.get_escrow();
    assert_eq!(batch_updated_escrow.milestones.get(0).unwrap().status, String::from_str(&env, "reviewed"));
    assert_eq!(batch_updated_escrow.milestones.get(1).unwrap().status, String::from_str(&env, "reviewed"));
    assert_eq!(
        batch_updated_escrow.milestones.get(0).unwrap().evidence,
        String::from_str(&env, "Batch update evidence")
    );
    assert_eq!(
        batch_updated_escrow.milestones.get(1).unwrap().evidence,
        String::from_str(&env, "Batch update evidence")
    );

    // Test with negative index
    let negative_update = vec![
        &env,
        MilestoneUpdate {
            index: -1,
            status: String::from_str(&env, "reviewed"),
            evidence: Some(String::from_str(&env, "Batch update evidence")),
        },
    ];
    let result = escrow_approver.try_change_milestone_status(
        &negative_update,
        &service_provider_address,
    );
    assert!(result.is_err());

    // Test with empty status
    let empty_status_update = vec![
        &env,
        MilestoneUpdate {
            index: 0,
            status: String::from_str(&env, ""),
            evidence: Some(String::from_str(&env, "Batch update evidence")),
        },
    ];
    let result = escrow_approver.try_change_milestone_status(
        &empty_status_update,
        &service_provider_address,
    );
    assert!(result.is_err());

    // Test with different status and evidence for each milestone
    let different_updates = vec![
        &env,
        MilestoneUpdate {
            index: 0,
            status: String::from_str(&env, "completed"),
            evidence: Some(String::from_str(&env, "Evidence for milestone 0")),
        },
        MilestoneUpdate {
            index: 1,
            status: String::from_str(&env, "in-progress"),
            evidence: None,
        },
    ];
    
    escrow_approver.change_milestone_status(
        &different_updates,
        &service_provider_address,
    );

    let final_check_escrow = escrow_approver.get_escrow();
    assert_eq!(final_check_escrow.milestones.get(0).unwrap().status, String::from_str(&env, "completed"));
    assert_eq!(final_check_escrow.milestones.get(1).unwrap().status, String::from_str(&env, "in-progress"));
    assert_eq!(
        final_check_escrow.milestones.get(0).unwrap().evidence,
        String::from_str(&env, "Evidence for milestone 0")
    );
    // Milestone 1 should keep its previous evidence since we passed None
    assert_eq!(
        final_check_escrow.milestones.get(1).unwrap().evidence,
        String::from_str(&env, "Batch update evidence")
    );
}

#[test]
fn test_release_funds_successful_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let _receiver_address = Address::generate(&env);
    let trustless_work_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let amount: i128 = 100_000_000;
    usdc_token.1.mint(&approver_address, &(amount as i128));

    let platform_fee = 5 * 100;

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Completed"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: false,
        },
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Completed"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: false,
        },
    ];

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        receiver: _receiver_address.clone(),
        observers: vec![&env],
    };

    let flags: Flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let engagement_id = String::from_str(&env, "test_escrow_1");
    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles,
        amount: amount,
        platform_fee: platform_fee,
        milestones: milestones.clone(),
        flags,
        trustline,
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&escrow_properties);

    usdc_token
        .1
        .mint(&escrow_approver.address, &(amount as i128));

    let milestone_indices = vec![&env, 0, 1];
    escrow_approver.approve_milestones(&milestone_indices, &approver_address);
    escrow_approver.release_funds(&release_signer_address, &trustless_work_address);

    let total_amount = amount as i128;
    let trustless_work_commission = ((total_amount * 30) / 10000) as i128;
    let platform_commission = (total_amount * platform_fee as i128) / 10000 as i128;
    let receiver_amount =
        (total_amount - (trustless_work_commission + platform_commission)) as i128;

    assert_eq!(
        usdc_token.0.balance(&trustless_work_address),
        trustless_work_commission,
        "Trustless Work commission amount is incorrect"
    );

    assert_eq!(
        usdc_token.0.balance(&platform_address),
        platform_commission,
        "Platform commission amount is incorrect"
    );

    assert_eq!(
        usdc_token.0.balance(&_receiver_address),
        receiver_amount,
        "Receiver received incorrect amount"
    );

    assert_eq!(
        usdc_token.0.balance(&service_provider_address),
        0,
        "Service Provider should have zero balance when using separate receiver"
    );

    assert_eq!(
        usdc_token.0.balance(&escrow_approver.address),
        0,
        "Contract should have zero balance after claiming earnings"
    );
}

// Scenario 2: Milestones incomplete
#[test]
fn test_release_funds_milestones_incomplete() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let _receiver_address = Address::generate(&env);
    let trustless_work_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let engagement_id_incomplete_milestones = String::from_str(&env, "test_incomplete_milestones");
    let amount: i128 = 100_000_000;
    let platform_fee = 3 * 100;

    let incomplete_milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Completed"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: false,
        },
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: false, // Not approved yet
        },
    ];

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        receiver: service_provider_address.clone(),
        observers: vec![&env],
    };

    let flags: Flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id_incomplete_milestones.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles,
        amount: amount,
        platform_fee: platform_fee,
        milestones: incomplete_milestones.clone(),
        flags,
        trustline,
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&escrow_properties);

    usdc_token
        .1
        .mint(&escrow_approver.address, &(amount as i128));
    let milestone_indices = vec![&env, 0];
    escrow_approver.approve_milestones(&milestone_indices, &approver_address);
    // Try to distribute earnings with incomplete milestones (should fail)
    let result =
        escrow_approver.try_release_funds(&release_signer_address, &trustless_work_address);
    assert!(result.is_err());
}

#[test]
fn test_release_funds_same_receiver_as_provider() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    // Use service_provider_address as receiver to test same-address case
    let _receiver_address = service_provider_address.clone();
    let trustless_work_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let amount: i128 = 100_000_000;
    usdc_token.1.mint(&approver_address, &(amount as i128));

    let platform_fee = 5 * 100;

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Completed"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: false,
        },
    ];

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        receiver: _receiver_address.clone(), // Set to service_provider to test same-address case
        observers: vec![&env],
    };

    let flags: Flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let engagement_id = String::from_str(&env, "test_escrow_same_receiver");
    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles,
        amount: amount,
        platform_fee: platform_fee,
        milestones: milestones.clone(),
        flags,
        trustline,
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&escrow_properties);

    usdc_token
        .1
        .mint(&escrow_approver.address, &(amount as i128));

    let milestone_indices = vec![&env, 0];
    escrow_approver.approve_milestones(&milestone_indices, &approver_address);
    escrow_approver.release_funds(&release_signer_address, &trustless_work_address);

    let total_amount = amount as i128;
    let trustless_work_commission = ((total_amount * 30) / 10000) as i128;
    let platform_commission = (total_amount * platform_fee as i128) / 10000 as i128;
    let service_provider_amount =
        (total_amount - (trustless_work_commission + platform_commission)) as i128;

    assert_eq!(
        usdc_token.0.balance(&trustless_work_address),
        trustless_work_commission,
        "Trustless Work commission amount is incorrect"
    );

    assert_eq!(
        usdc_token.0.balance(&platform_address),
        platform_commission,
        "Platform commission amount is incorrect"
    );

    assert_eq!(
        usdc_token.0.balance(&service_provider_address),
        service_provider_amount,
        "Service Provider should receive funds when receiver is set to same address"
    );

    assert_eq!(
        usdc_token.0.balance(&escrow_approver.address),
        0,
        "Contract should have zero balance after claiming earnings"
    );
}

#[test]
fn test_release_funds_invalid_receiver_fallback() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let trustless_work_address = Address::generate(&env);

    // Create a valid but separate receiver address
    let _receiver_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let amount: i128 = 100_000_000;
    usdc_token.1.mint(&approver_address, &(amount as i128));

    let platform_fee = 5 * 100;

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Completed"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: false,
        },
    ];

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        receiver: _receiver_address.clone(), // Different receiver address than service provider
        observers: vec![&env],
    };

    let flags: Flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let engagement_id = String::from_str(&env, "test_escrow_receiver");
    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles,
        amount: amount,
        platform_fee: platform_fee,
        milestones: milestones.clone(),
        flags,
        trustline,
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&escrow_properties);

    usdc_token
        .1
        .mint(&escrow_approver.address, &(amount as i128));

    let milestone_indices = vec![&env, 0];
    escrow_approver.approve_milestones(&milestone_indices, &approver_address);
    escrow_approver.release_funds(&release_signer_address, &trustless_work_address);

    let total_amount = amount as i128;
    let trustless_work_commission = ((total_amount * 30) / 10000) as i128;
    let platform_commission = (total_amount * platform_fee as i128) / 10000 as i128;
    let receiver_amount =
        (total_amount - (trustless_work_commission + platform_commission)) as i128;

    assert_eq!(
        usdc_token.0.balance(&trustless_work_address),
        trustless_work_commission,
        "Trustless Work commission amount is incorrect"
    );

    assert_eq!(
        usdc_token.0.balance(&platform_address),
        platform_commission,
        "Platform commission amount is incorrect"
    );

    // Funds should go to the receiver (not service provider)
    assert_eq!(
        usdc_token.0.balance(&_receiver_address),
        receiver_amount,
        "Receiver should receive funds when set to a different address than service provider"
    );

    // The service provider should not receive funds when a different receiver is set
    assert_eq!(
        usdc_token.0.balance(&service_provider_address),
        0,
        "Service provider should not receive funds when a different receiver is set"
    );

    assert_eq!(
        usdc_token.0.balance(&escrow_approver.address),
        0,
        "Contract should have zero balance after claiming earnings"
    );
}

#[test]
fn test_dispute_management() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let trustless_work_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let engagement_id = String::from_str(&env, "test_dispute");
    let amount: i128 = 100_000_000;
    let platform_fee = 3 * 100;

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: false,
        },
    ];

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        receiver: service_provider_address.clone(),
        observers: vec![&env],
    };

    let flags: Flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles,
        amount: amount,
        platform_fee: platform_fee,
        milestones: milestones.clone(),
        flags,
        trustline,
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&escrow_properties);

    let escrow = escrow_approver.get_escrow();
    assert!(!escrow.flags.disputed);

    escrow_approver.dispute_escrow(&approver_address);

    let escrow_after_change = escrow_approver.get_escrow();
    assert!(escrow_after_change.flags.disputed);

    usdc_token.1.mint(&approver_address, &(amount as i128));
    // Test block on distributing earnings during dispute
    let result =
        escrow_approver.try_release_funds(&release_signer_address, &trustless_work_address);
    assert!(result.is_err());

    let _ = escrow_approver.try_dispute_escrow(&approver_address);

    let escrow_after_second_change = escrow_approver.get_escrow();
    assert!(escrow_after_second_change.flags.disputed);
}

#[test]
fn test_dispute_resolution_process() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let trustless_work_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let amount: i128 = 100_000_000;
    usdc_token.1.mint(&approver_address, &(amount as i128));

    let platform_fee = 5 * 100;

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Completed"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: false,
        },
    ];

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        receiver: service_provider_address.clone(),
        observers: vec![&env],
    };

    let flags: Flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let engagement_id = String::from_str(&env, "test_dispute_resolution");
    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles,
        amount: amount,
        platform_fee: platform_fee,
        milestones: milestones.clone(),
        flags,
        trustline,
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&escrow_properties);

    usdc_token
        .0
        .transfer(&approver_address, &escrow_approver.address, &amount);

    escrow_approver.dispute_escrow(&approver_address);

    let escrow_with_dispute = escrow_approver.get_escrow();
    assert!(escrow_with_dispute.flags.disputed);

    // Try to resolve dispute with incorrect dispute resolver (should fail)
    let mut wrong_dist = Map::new(&env);
    wrong_dist.set(approver_address.clone(), 50_000_000);
    wrong_dist.set(service_provider_address.clone(), 50_000_000);
    let result = escrow_approver.try_resolve_dispute(
        &approver_address,
        &trustless_work_address,
        &wrong_dist,
    );
    assert!(result.is_err());

    let approver_funds: i128 = 50_000_000;
    let insufficient_receiver_funds: i128 = 40_000_000;

    let mut incorrect_dist = Map::new(&env);
    incorrect_dist.set(approver_address.clone(), approver_funds);
    incorrect_dist.set(
        service_provider_address.clone(),
        insufficient_receiver_funds,
    );
    let incorrect_dispute_resolution_result = escrow_approver.try_resolve_dispute(
        &dispute_resolver_address,
        &trustless_work_address,
        &incorrect_dist,
    );

    assert!(incorrect_dispute_resolution_result.is_err());

    let empty_dist = Map::new(&env);
    let dispute_resolution_with_incorrect_funds = escrow_approver.try_resolve_dispute(
        &dispute_resolver_address,
        &trustless_work_address,
        &empty_dist,
    );

    assert!(dispute_resolution_with_incorrect_funds.is_err());

    // Resolve dispute with correct dispute resolver (50/50 split)
    let receiver_funds: i128 = 50_000_000;

    let mut ok_dist = Map::new(&env);
    ok_dist.set(approver_address.clone(), approver_funds);
    ok_dist.set(service_provider_address.clone(), receiver_funds);
    escrow_approver.resolve_dispute(&dispute_resolver_address, &trustless_work_address, &ok_dist);

    // Verify dispute was resolved
    let escrow_after_resolution = escrow_approver.get_escrow();
    assert!(!escrow_after_resolution.flags.disputed);
    assert!(escrow_after_resolution.flags.resolved);

    let total_amount = amount as i128;
    let trustless_work_commission = ((total_amount * 30) / 10000) as i128;
    let platform_commission = (total_amount * platform_fee as i128) / 10000 as i128;
    let remaining_amount = total_amount - (trustless_work_commission + platform_commission);

    let platform_amount = platform_commission;
    let service_provider_amount = (remaining_amount * receiver_funds) / total_amount;
    let approver_amount = (remaining_amount * approver_funds) / total_amount;

    // Check balances
    assert_eq!(
        usdc_token.0.balance(&trustless_work_address),
        trustless_work_commission,
        "Trustless Work commission amount is incorrect"
    );

    assert_eq!(
        usdc_token.0.balance(&platform_address),
        platform_amount,
        "Platform commission amount is incorrect"
    );

    assert_eq!(
        usdc_token.0.balance(&service_provider_address),
        service_provider_amount,
        "Service provider amount is incorrect"
    );

    assert_eq!(
        usdc_token.0.balance(&approver_address),
        approver_amount,
        "Approver amount is incorrect"
    );
}

#[test]
fn test_fund_escrow_successful_deposit() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let _receiver_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let amount: i128 = 100_000_000;
    usdc_token.1.mint(&approver_address, &amount);

    let platform_fee = 5 * 100;

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: false,
        },
    ];

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        receiver: _receiver_address.clone(),
        observers: vec![&env],
    };

    let flags: Flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let engagement_id = String::from_str(&env, "test_escrow_fund");
    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles,
        amount: amount,
        platform_fee: platform_fee,
        milestones: milestones.clone(),
        flags,
        trustline,
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&escrow_properties);

    // Check initial balances
    assert_eq!(usdc_token.0.balance(&approver_address), amount);
    assert_eq!(usdc_token.0.balance(&escrow_approver.address), 0);

    let deposit_amount = amount / 2;

    let test_fund = escrow_approver.try_fund_escrow(&approver_address, &escrow_properties, &0);
    assert!(test_fund.is_err());

    escrow_approver.fund_escrow(&approver_address, &escrow_properties, &deposit_amount);

    // Check balances after deposit
    assert_eq!(
        usdc_token.0.balance(&approver_address),
        amount - deposit_amount
    );
    assert_eq!(
        usdc_token.0.balance(&escrow_approver.address),
        deposit_amount
    );

    // Deposit remaining amount
    escrow_approver.fund_escrow(&approver_address, &escrow_properties, &deposit_amount);

    assert_eq!(usdc_token.0.balance(&approver_address), 0);
    assert_eq!(usdc_token.0.balance(&escrow_approver.address), amount);
}

#[test]
fn test_fund_escrow_signer_insufficient_funds_error() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let _receiver_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let amount: i128 = 100_000_000;
    // Only mint a small amount to the approver
    let small_amount: i128 = 1_000_000;
    usdc_token.1.mint(&approver_address, &small_amount);

    let platform_fee = 5 * 100;

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: false,
        },
    ];

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        receiver: _receiver_address.clone(),
        observers: vec![&env],
    };

    let flags: Flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let engagement_id = String::from_str(&env, "test_escrow_insufficient_funds");
    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles,
        amount: amount,
        platform_fee: platform_fee,
        milestones: milestones.clone(),
        flags,
        trustline,
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&escrow_properties);

    // Check initial balance
    assert_eq!(usdc_token.0.balance(&approver_address), small_amount);

    // Try to deposit more than the approver has (should fail)
    let result = escrow_approver.try_fund_escrow(&approver_address, &escrow_properties, &amount);
    assert!(result.is_err());

    // Verify balances didn't change
    assert_eq!(usdc_token.0.balance(&approver_address), small_amount);
    assert_eq!(usdc_token.0.balance(&escrow_approver.address), 0);
}

#[test]
fn test_dispute_escrow_authorized_and_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);
    let receiver = Address::generate(&env);
    let unauthorized = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let roles = Roles {
        approver: approver.clone(),
        service_provider: service_provider.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer.clone(),
        dispute_resolver: dispute_resolver.clone(),
        receiver: receiver.clone(),
        observers: vec![&env],
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
        engagement_id: String::from_str(&env, "engagement_001"),
        title: String::from_str(&env, "Escrow for test"),
        description: String::from_str(&env, "Test for dispute flag"),
        roles,
        amount: 10_000_000,
        platform_fee: 0,
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

    let test_data = create_escrow_contract(&env);
    let escrow_client_1 = test_data.client;

    escrow_client_1.initialize_escrow(&escrow_base);
    escrow_client_1.dispute_escrow(&approver);

    let updated_escrow = escrow_client_1.get_escrow();
    assert!(
        updated_escrow.flags.disputed,
        "Dispute flag should be set to true for authorized address"
    );

    let test_data = create_escrow_contract(&env);
    let escrow_client_2 = test_data.client;

    escrow_client_2.initialize_escrow(&escrow_base);
    let result = escrow_client_2.try_dispute_escrow(&unauthorized);

    assert!(
        result.is_err(),
        "Unauthorized user should not be able to change dispute flag"
    );
}

#[test]
fn test_get_multiple_escrow_balances_platform_authorized() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);
    let receiver = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let roles = Roles {
        approver: approver.clone(),
        service_provider: service_provider.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer.clone(),
        dispute_resolver: dispute_resolver.clone(),
        receiver: receiver.clone(),
        observers: vec![&env],
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
fn test_approve_multiple_milestones() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let amount: i128 = 1000;
    let platform_fee: u32 = 300;

    let usdc_token = create_usdc_token(&env, &admin);

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: false,
        },
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: false,
        },
        Milestone {
            description: String::from_str(&env, "Third milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: false,
        },
    ];

    let engagement_id = String::from_str(&env, "test-multiple-milestones");

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        receiver: service_provider_address.clone(),
        observers: vec![&env],
    };

    let flags: Flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Multiple Milestones"),
        description: String::from_str(&env, "Test approving multiple milestones at once"),
        roles,
        amount,
        platform_fee,
        milestones: milestones.clone(),
        flags,
        trustline,
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&escrow_properties);

    usdc_token
        .1
        .mint(&escrow_approver.address, &amount);

    // Test 1: Aprobar múltiples milestones a la vez (0 y 1)
    let milestone_indices = vec![&env, 0, 1];
    escrow_approver.approve_milestones(&milestone_indices, &approver_address);

    let escrow_after_approval = escrow_approver.get_escrow();
    assert!(escrow_after_approval.milestones.get(0).unwrap().approved, "Milestone 0 should be approved");
    assert!(escrow_after_approval.milestones.get(1).unwrap().approved, "Milestone 1 should be approved");
    assert!(!escrow_after_approval.milestones.get(2).unwrap().approved, "Milestone 2 should not be approved");

    // Test 2: Aprobar el último milestone
    let milestone_indices = vec![&env, 2];
    escrow_approver.approve_milestones(&milestone_indices, &approver_address);

    let escrow_after_all_approved = escrow_approver.get_escrow();
    assert!(escrow_after_all_approved.milestones.get(2).unwrap().approved, "Milestone 2 should be approved");

    // Test 3: Intentar aprobar con un índice negativo (debe fallar)
    let negative_indices = vec![&env, -1];
    let result = escrow_approver.try_approve_milestones(&negative_indices, &approver_address);
    assert!(result.is_err(), "Should fail with negative index");

    // Test 4: Intentar aprobar con un índice que no existe (debe fallar)
    let invalid_indices = vec![&env, 10];
    let result = escrow_approver.try_approve_milestones(&invalid_indices, &approver_address);
    assert!(result.is_err(), "Should fail with non-existent index");

    // Test 5: Intentar aprobar múltiples índices donde uno es inválido (debe fallar)
    let mixed_indices = vec![&env, 0, 99];
    let result = escrow_approver.try_approve_milestones(&mixed_indices, &approver_address);
    assert!(result.is_err(), "Should fail when any index is invalid");

    // Test 6: Intentar aprobar con un índice negativo en un conjunto de índices (debe fallar)
    let mixed_negative_indices = vec![&env, 0, -5];
    let result = escrow_approver.try_approve_milestones(&mixed_negative_indices, &approver_address);
    assert!(result.is_err(), "Should fail when any index is negative");
}

#[test]
fn test_milestone_index_overflow_protection() {
    let env = Env::default();
    env.mock_all_auths();

    let approver_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let platform_fee = 3 * 100;
    let amount: i128 = 100_000_000;
    
    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: false,
        },
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: false,
        },
    ];

    let usdc_token = create_usdc_token(&env, &admin);
    let engagement_id = String::from_str(&env, "test-overflow");

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        receiver: service_provider_address.clone(),
        observers: vec![&env],
    };

    let flags: Flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let escrow_properties = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Overflow Protection"),
        description: String::from_str(&env, "Should reject large i128 values"),
        roles: roles.clone(),
        amount,
        platform_fee,
        milestones: milestones.clone(),
        flags: flags.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_client = test_data.client;

    escrow_client.initialize_escrow(&escrow_properties);

    // Test 1: Intentar aprobar con índice que causaría overflow (2^32)
    let overflow_index: i128 = 4_294_967_296; // 2^32, would wrap to 0 if not validated
    let overflow_indices = vec![&env, overflow_index];
    let result = escrow_client.try_approve_milestones(&overflow_indices, &approver_address);
    assert!(result.is_err(), "Should fail with index >= 2^32");

    // Test 2: Intentar aprobar con índice muy grande
    let large_index: i128 = i128::MAX;
    let large_indices = vec![&env, large_index];
    let result = escrow_client.try_approve_milestones(&large_indices, &approver_address);
    assert!(result.is_err(), "Should fail with very large index");

    // Test 3: Intentar actualizar milestone status con índice overflow
    let overflow_update = vec![
        &env,
        MilestoneUpdate {
            index: 4_294_967_296, // 2^32
            status: String::from_str(&env, "completed"),
            evidence: Some(String::from_str(&env, "Evidence")),
        },
    ];
    let result = escrow_client.try_change_milestone_status(
        &overflow_update,
        &service_provider_address,
    );
    assert!(result.is_err(), "Should fail when updating with overflow index");

    // Test 4: Intentar actualizar con índice muy grande
    let large_update = vec![
        &env,
        MilestoneUpdate {
            index: i128::MAX,
            status: String::from_str(&env, "completed"),
            evidence: Some(String::from_str(&env, "Evidence")),
        },
    ];
    let result = escrow_client.try_change_milestone_status(
        &large_update,
        &service_provider_address,
    );
    assert!(result.is_err(), "Should fail when updating with very large index");

    // Test 5: Verificar que índices válidos todavía funcionan correctamente
    let valid_indices = vec![&env, 0, 1];
    escrow_client.approve_milestones(&valid_indices, &approver_address);
    
    let updated_escrow = escrow_client.get_escrow();
    assert!(updated_escrow.milestones.get(0).unwrap().approved, "Valid index 0 should work");
    assert!(updated_escrow.milestones.get(1).unwrap().approved, "Valid index 1 should work");
}

#[test]
fn test_withdraw_remaining_funds_with_fees() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let trustless_work_address = Address::generate(&env);
    let recipient1 = Address::generate(&env);
    let recipient2 = Address::generate(&env);

    let amount: i128 = 1_000_000_000; // 1000 USDC (7 decimals)
    let platform_fee: u32 = 300; // 3%

    let usdc_token = create_usdc_token(&env, &admin);

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: false,
        },
    ];

    let engagement_id = String::from_str(&env, "test-withdraw-funds");

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        receiver: service_provider_address.clone(),
        observers: vec![&env],
    };

    let flags: Flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Withdraw Remaining Funds"),
        description: String::from_str(&env, "Test withdraw with fee calculation"),
        roles,
        amount,
        platform_fee,
        milestones: milestones.clone(),
        flags,
        trustline,
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_contract = test_data.client;

    escrow_contract.initialize_escrow(&escrow_properties);

    // Fund the escrow - mint to approver first, then transfer via fund_escrow
    usdc_token.1.mint(&approver_address, &amount);
    escrow_contract.fund_escrow(&approver_address, &escrow_properties, &amount);

    // Update milestone status to complete
    let milestone_update = vec![
        &env,
        MilestoneUpdate {
            index: 0,
            status: String::from_str(&env, "Completed"),
            evidence: Some(String::from_str(&env, "Work completed")),
        },
    ];
    escrow_contract.change_milestone_status(&milestone_update, &service_provider_address);

    // Approve milestone
    let milestone_indices = vec![&env, 0];
    escrow_contract.approve_milestones(&milestone_indices, &approver_address);

    // Release funds
    escrow_contract.release_funds(&release_signer_address, &trustless_work_address);

    // Get the current contract balance after release
    let balance_after_release = usdc_token.0.balance(&escrow_contract.address);

    // Simulate remaining balance (contract still has some funds left)
    let remaining_amount: i128 = 100_000_000; // 100 USDC remaining
    usdc_token.1.mint(&escrow_contract.address, &remaining_amount);

    // Get initial balances
    let initial_tw_balance = usdc_token.0.balance(&trustless_work_address);
    let initial_platform_balance = usdc_token.0.balance(&platform_address);
    let initial_recipient1_balance = usdc_token.0.balance(&recipient1);
    let initial_recipient2_balance = usdc_token.0.balance(&recipient2);

    // Setup distribution: 60 USDC to recipient1, 40 USDC to recipient2
    let recipient1_share: i128 = 60_000_000;
    let recipient2_share: i128 = 40_000_000;

    let mut distributions: Map<Address, i128> = Map::new(&env);
    distributions.set(recipient1.clone(), recipient1_share);
    distributions.set(recipient2.clone(), recipient2_share);

    // Calculate expected fees manually
    // Trustless Work Fee: 0.3% (30 bps) of total = 100 * 0.003 = 0.3 USDC = 300_000
    // Platform Fee: 3% (300 bps) of total = 100 * 0.03 = 3 USDC = 3_000_000
    // Total Fees: 3_300_000
    // Net Amount: 100_000_000 - 3_300_000 = 96_700_000

    let expected_tw_fee: i128 = 300_000; // 0.3 USDC
    let expected_platform_fee: i128 = 3_000_000; // 3 USDC
    let _expected_total_fees: i128 = expected_tw_fee + expected_platform_fee;

    // For recipient1 (60% of distribution):
    // Fee share = (60_000_000 * 3_300_000) / 100_000_000 = 1_980_000
    // Net amount = 60_000_000 - 1_980_000 = 58_020_000
    let expected_recipient1_net: i128 = 58_020_000;

    // For recipient2 (40% of distribution):
    // Fee share = (40_000_000 * 3_300_000) / 100_000_000 = 1_320_000
    // Net amount = 40_000_000 - 1_320_000 = 38_680_000
    let expected_recipient2_net: i128 = 38_680_000;

    // Execute withdraw_remaining_funds
    escrow_contract.withdraw_remaining_funds(
        &dispute_resolver_address,
        &trustless_work_address,
        &distributions,
    );

    // Verify final balances
    let final_tw_balance = usdc_token.0.balance(&trustless_work_address);
    let final_platform_balance = usdc_token.0.balance(&platform_address);
    let final_recipient1_balance = usdc_token.0.balance(&recipient1);
    let final_recipient2_balance = usdc_token.0.balance(&recipient2);
    let final_contract_balance = usdc_token.0.balance(&escrow_contract.address);

    // Assert Trustless Work received correct fee (initial from release + from withdraw)
    let tw_fee_from_withdraw = final_tw_balance - initial_tw_balance;
    assert_eq!(
        tw_fee_from_withdraw,
        expected_tw_fee,
        "Trustless Work should receive 0.3% fee from withdrawal"
    );

    // Assert Platform received correct fee (initial from release + from withdraw)
    let platform_fee_from_withdraw = final_platform_balance - initial_platform_balance;
    assert_eq!(
        platform_fee_from_withdraw,
        expected_platform_fee,
        "Platform should receive 3% fee from withdrawal"
    );

    // Assert recipient1 received net amount after proportional fee deduction
    assert_eq!(
        final_recipient1_balance - initial_recipient1_balance,
        expected_recipient1_net,
        "Recipient1 should receive net amount after fees"
    );

    // Assert recipient2 received net amount after proportional fee deduction
    assert_eq!(
        final_recipient2_balance - initial_recipient2_balance,
        expected_recipient2_net,
        "Recipient2 should receive net amount after fees"
    );

    // Assert contract balance equals what was left after the release
    assert_eq!(
        final_contract_balance,
        balance_after_release,
        "Contract should have the original remaining balance after withdrawal of extra funds"
    );

    // Verify total withdrawn amounts add up correctly
    let total_withdrawn = tw_fee_from_withdraw
        + platform_fee_from_withdraw
        + (final_recipient1_balance - initial_recipient1_balance)
        + (final_recipient2_balance - initial_recipient2_balance);

    assert_eq!(
        total_withdrawn,
        remaining_amount,
        "Total withdrawn should equal the extra remaining amount"    );
}

#[test]
fn test_dispute_resolution_rounding_edge_case() {
    let env = Env::default();
    env.mock_all_auths();
    
    // Setup with small amounts that trigger rounding
    // Total = 2.
    // Trustless fee = 2 * 30 / 10000 = 0.
    // Platform fee = 50% = 2 * 5000 / 10000 = 1.
    // Total fees = 1.
    // Distributions: A=1, B=1.
    // Share A: (1 * 1) / 2 = 0. Net A = 1.
    // Share B: (1 * 1) / 2 = 0. Net B = 1.
    // Total Net to distribute = 2.
    // Remaining balance after fees = 2 - 1 = 1.
    // Fails on second transfer.

    let total_amount: i128 = 2;
    let platform_fee = 50 * 100; // 50%

    let approver_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let receiver_a = Address::generate(&env);
    let receiver_b = Address::generate(&env);
    
    // Create token
    let (token_client, token_admin) = create_usdc_token(&env, &admin);
    let trustline: Trustline = Trustline {
        address: token_client.address.clone(),
    };

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: false,
        },
    ];

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        receiver: service_provider_address.clone(),
        observers: vec![&env],
    };

    let flags: Flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
    };

    let engagement_id = String::from_str(&env, "test_rounding");
    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Rounding"),
        description: String::from_str(&env, "Test Rounding Description"),
        roles: roles.clone(),
        amount: total_amount,
        platform_fee: platform_fee,
        milestones: milestones.clone(),
        flags: flags.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let client = test_data.client;
    let contract_address = client.address.clone();

    // Initialize
    client.initialize_escrow(&escrow_properties);

    // Fund
    token_admin.mint(&approver_address, &total_amount);
    client.fund_escrow(&approver_address, &escrow_properties, &total_amount);

    // Dispute
    client.dispute_escrow(&approver_address);

    let mut distributions = Map::new(&env);
    distributions.set(receiver_a.clone(), 1);
    distributions.set(receiver_b.clone(), 1);
    
    // We also need trustless work address. 
    // In `resolve_dispute(e, dispute_resolver, trustless_work_address, distributions)`
    let trustless_work_address = Address::generate(&env);

    // This is expected to fail with the current bug
    let result = client.try_resolve_dispute(
        &dispute_resolver_address,
        &trustless_work_address,
        &distributions
    );
    
    assert!(result.is_ok(), "Should deal with rounding errors without reverting");

    // Verify final balance is 0
    let final_balance = token_client.balance(&contract_address);
    assert_eq!(final_balance, 0, "All funds should be distributed");
}