#![cfg(test)]

extern crate std;

use crate::contract::EscrowContract;
use crate::contract::EscrowContractClient;
use crate::storage::types::{Escrow, Flags, Milestone, Roles, Trustline};

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

fn create_escrow_contract<'a>(env: &Env) -> TestData {
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
    escrow_client.approve_milestone(&0, &approver_address);
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
    let new_status = String::from_str(&env, "completed");
    let new_evidence = Some(String::from_str(&env, "New evidence"));
    escrow_approver.change_milestone_status(
        &(0 as i128),
        &new_status,
        &new_evidence,
        &service_provider_address,
    );

    let updated_escrow = escrow_approver.get_escrow();
    assert_eq!(updated_escrow.milestones.get(0).unwrap().status, new_status);
    assert_eq!(
        updated_escrow.milestones.get(0).unwrap().evidence,
        String::from_str(&env, "New evidence")
    );

    // Change milestone approved (valid case)
    escrow_approver.approve_milestone(&(0 as i128), &approver_address);

    let final_escrow = escrow_approver.get_escrow();
    assert!(final_escrow.milestones.get(0).unwrap().approved);

    let invalid_index = 10 as i128;
    let new_status = String::from_str(&env, "completed");
    let new_evidence = Some(String::from_str(&env, "New evidence"));

    let result = escrow_approver.try_change_milestone_status(
        &invalid_index,
        &new_status,
        &new_evidence,
        &service_provider_address,
    );
    assert!(result.is_err());

    let result = escrow_approver.try_approve_milestone(&invalid_index, &approver_address);
    assert!(result.is_err());

    let unauthorized_address = Address::generate(&env);

    // Test for `change_status` by invalid service provider
    let result = escrow_approver.try_change_milestone_status(
        &(0 as i128),
        &new_status,
        &new_evidence,
        &unauthorized_address,
    );
    assert!(result.is_err());

    // Test for `change_approved` by invalid approver
    let result = escrow_approver.try_approve_milestone(&(0 as i128), &unauthorized_address);
    assert!(result.is_err());
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

    escrow_approver.approve_milestone(&0, &approver_address);
    escrow_approver.approve_milestone(&1, &approver_address);
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
    escrow_approver.approve_milestone(&0, &approver_address);
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

    escrow_approver.approve_milestone(&0, &approver_address);
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

    escrow_approver.approve_milestone(&0, &approver_address);
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
