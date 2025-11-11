#![cfg(test)]

extern crate std;

use crate::storage::types::{ Escrow, Flags, Milestone, Roles, Trustline };
use crate::contract::EscrowContract;
use crate::contract::EscrowContractClient;

use soroban_sdk::Vec;
use soroban_sdk::{ testutils::Address as _, vec, Address, Env, String, token };
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
    client:  EscrowContractClient<'a>,
}

fn create_escrow_contract<'a>(env: &Env) -> TestData {
    env.mock_all_auths();
    let admin = Address::generate(env);
    let client = EscrowContractClient::new(
        env,
        &env.register(
            EscrowContract {},
            (
                admin.clone(),
            )
        ),
    );

    TestData {
        client,
    }
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
    let platform_fee = 3;
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
        decimals: 10_000_000,
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
    let platform_fee = (0.3 * 10i128.pow(18) as f64) as i128;

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
        decimals: 10_000_000,
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
    let result = escrow_approver
        .try_update_escrow(&non_platform_address, &updated_escrow_properties);
    assert!(result.is_err());
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
    let platform_fee = (0.3 * 10i128.pow(18) as f64) as i128;

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
        decimals: 10_000_000,
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
    escrow_approver.approve_milestone(&(0 as i128), &true, &approver_address);

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

    let result =
        escrow_approver.try_approve_milestone(&invalid_index, &true, &approver_address);
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
    let result =
        escrow_approver.try_approve_milestone(&(0 as i128), &true, &unauthorized_address);
    assert!(result.is_err());

    let test_data = create_escrow_contract(&env);
    let new_escrow_approver = test_data.client;

    //Escrow Test with no milestone
    let escrow_properties_v2: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Updated Escrow"),
        description: String::from_str(&env, "Updated Escrow Description"),
        roles,
        amount: amount,
        platform_fee: platform_fee,
        milestones: vec![&env],
        flags,
        trustline,
        receiver_memo: 0,
    };

    new_escrow_approver.initialize_escrow(&escrow_properties_v2);

    let result = new_escrow_approver.try_change_milestone_status(
        &(0 as i128),
        &new_status,
        &new_evidence,
        &service_provider_address,
    );
    assert!(result.is_err());

    let result =
        new_escrow_approver.try_approve_milestone(&(0 as i128), &true, &approver_address);
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
    let trustless_work_address = Address::generate(&env);
    let _receiver_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let amount: i128 = 100_000_000;
    usdc_token.1.mint(&approver_address, &(amount as i128));

    let platform_fee = 500;

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Completed"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: true,
        },
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Completed"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: true,
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
        decimals: 10_000_000,
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

    usdc_token.1.mint(&escrow_approver.address, &(amount as i128));

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

//test claim escrow earnings in failure scenarios
// Scenario 1: Escrow with no milestones:
#[test]
fn test_release_funds_no_milestones() {
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

    let engagement_id_no_milestones = String::from_str(&env, "test_no_milestones");
    let amount: i128 = 100_000_000;
    let platform_fee = 30;

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
        decimals: 10_000_000,
    };

    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id_no_milestones.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles,
        amount: amount,
        platform_fee: platform_fee,
        milestones: vec![&env],
        flags,
        trustline,
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&escrow_properties);

    usdc_token.1.mint(&escrow_approver.address, &(amount as i128));

    // Try to claim earnings with no milestones (should fail)
    let result = escrow_approver.try_release_funds(&release_signer_address, &platform_address);
    assert!(result.is_err());
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

    let usdc_token = create_usdc_token(&env, &admin);

    let engagement_id_incomplete_milestones = String::from_str(&env, "test_incomplete_milestones");
    let amount: i128 = 100_000_000;
    let platform_fee = 30;

    // Define milestones with one not approved
    let incomplete_milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Completed"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: true,
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
        decimals: 10_000_000,
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

    usdc_token.1.mint(&escrow_approver.address, &(amount as i128));

    // Try to distribute earnings with incomplete milestones (should fail)
    let result = escrow_approver.try_release_funds(&release_signer_address, &platform_address);
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
    let trustless_work_address = Address::generate(&env);
    // Use service_provider_address as receiver to test same-address case
    let _receiver_address = service_provider_address.clone();

    let usdc_token = create_usdc_token(&env, &admin);

    let amount: i128 = 100_000_000;
    usdc_token.1.mint(&approver_address, &(amount as i128));

    let platform_fee = 500;

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Completed"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: true,
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
        decimals: 10_000_000,
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

    usdc_token.1.mint(&escrow_approver.address, &(amount as i128));

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

    let platform_fee = 500;

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Completed"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: true,
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
        decimals: 10_000_000,
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

    usdc_token.1.mint(&escrow_approver.address, &(amount as i128));

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

    let usdc_token = create_usdc_token(&env, &admin);


    let engagement_id = String::from_str(&env, "test_dispute");
    let amount: i128 = 100_000_000;
    let platform_fee = 30;

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
        decimals: 10_000_000,
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

    escrow_approver.dispute_escrow(&dispute_resolver_address);

    let escrow_after_change = escrow_approver.get_escrow();
    assert!(escrow_after_change.flags.disputed);

    usdc_token.1.mint(&approver_address, &(amount as i128));
    // Test block on distributing earnings during dispute
    let result = escrow_approver.try_release_funds(&release_signer_address, &platform_address);
    assert!(result.is_err());

    let _ = escrow_approver.try_dispute_escrow(&dispute_resolver_address);

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

    let platform_fee = 500;

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Completed"),
            evidence: String::from_str(&env, "Initial evidence"),
            approved: true,
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
        decimals: 10_000_000,
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

    usdc_token.0.transfer(&approver_address, &escrow_approver.address, &amount);

    escrow_approver.dispute_escrow(&approver_address);

    let escrow_with_dispute = escrow_approver.get_escrow();
    assert!(escrow_with_dispute.flags.disputed);

    // Try to resolve dispute with incorrect dispute resolver (should fail)
    let result = escrow_approver.try_resolve_dispute(
        &approver_address,
        &(50_000_000 as i128),
        &(50_000_000 as i128),
        &trustless_work_address,
    );
    assert!(result.is_err());

    let approver_funds: i128 = 50_000_000;
    let insufficient_receiver_funds: i128 = 40_000_000;

    let incorrect_dispute_resolution_result = escrow_approver.try_resolve_dispute(
        &dispute_resolver_address,
        &approver_funds,
        &insufficient_receiver_funds,
        &trustless_work_address,
    );

    assert!(incorrect_dispute_resolution_result.is_err());

    // Resolve dispute with correct dispute resolver (50/50 split)
    let receiver_funds: i128 = 50_000_000;

    escrow_approver.resolve_dispute(
        &dispute_resolver_address,
        &approver_funds,
        &receiver_funds,
        &trustless_work_address,
    );

    // Verify dispute was resolved
    let escrow_after_resolution = escrow_approver.get_escrow();
    assert!(!escrow_after_resolution.flags.disputed);
    assert!(escrow_after_resolution.flags.resolved);

    let total_amount = amount as i128;
    let trustless_work_commission = ((total_amount * 30) / 10000) as i128;
    let platform_commission = (total_amount * platform_fee as i128) / 10000 as i128;
    let remaining_amount = total_amount - (trustless_work_commission + platform_commission);

    let platform_amount = platform_commission;
    let trustless_amount = trustless_work_commission;
    let service_provider_amount = (remaining_amount * receiver_funds) / total_amount;
    let approver_amount = (remaining_amount * approver_funds) / total_amount;

    // Check balances
    assert_eq!(
        usdc_token.0.balance(&trustless_work_address),
        trustless_amount,
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

    let platform_fee = 500;

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
        decimals: 10_000_000,
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
    assert_eq!(usdc_token.0.balance(&escrow_approver.address ), 0);

    let deposit_amount = amount / 2;
    escrow_approver.fund_escrow(&approver_address, &deposit_amount);

    // Check balances after deposit
    assert_eq!(
        usdc_token.0.balance(&approver_address),
        amount - deposit_amount
    );
    assert_eq!(usdc_token.0.balance(&escrow_approver.address), deposit_amount);

    // Deposit remaining amount
    escrow_approver.fund_escrow(&approver_address, &deposit_amount);

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

    let platform_fee = 500;

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
        decimals: 10_000_000,
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
    let result = escrow_approver.try_fund_escrow(&approver_address, &amount);
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

    let escrow_base = Escrow {
        engagement_id: String::from_str(&env, "engagement_001"),
        title: String::from_str(&env, "Escrow for test"),
        description: String::from_str(&env, "Test for dispute flag"),
        roles,
        amount: 10_000_000,
        platform_fee: 0,
        milestones: Vec::new(&env),
        flags: Flags {
            disputed: false,
            released: false,
            resolved: false,
        },
        trustline: Trustline {
            address: usdc_token.0.address.clone(),
            decimals: 7,
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
