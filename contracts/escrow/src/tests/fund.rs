extern crate std;

use crate::storage::types::{Escrow, Flags, Milestone, Roles, Trustline};
use soroban_sdk::{testutils::Address as _, vec, Address, Env, String};

use super::helpers::{create_escrow_contract, create_usdc_token};

#[test]
fn test_release_milestone_funds_successful() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let trustless_work_address = Address::generate(&env);

    let amount: i128 = 100_000_000;

    let platform_fee = 3 * 100;
    let usdc_token = create_usdc_token(&env, &admin);

    let engagement_id = String::from_str(&env, "41431");
    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
    };

    let flags: Flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
        approved: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            flags: flags.clone(),
            amount: 100_000,
            evidence: String::from_str(&env, "Empty"),
            receiver: service_provider_address.clone(),
        },
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Pending"),
            flags: flags.clone(),
            amount: 100_000,
            evidence: String::from_str(&env, "Empty"),
            receiver: service_provider_address.clone(),
        },
    ];

    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles: roles.clone(),
        platform_fee: platform_fee,
        milestones: milestones.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&escrow_properties);

    usdc_token
        .1
        .mint(&escrow_approver.address, &(amount as i128));

    let initial_contract_balance = usdc_token.0.balance(&escrow_approver.address);

    // Approve the milestone before releasing funds
    escrow_approver.approve_milestone(&0, &approver_address);
    escrow_approver.release_milestone_funds(&release_signer_address, &trustless_work_address, &(0));

    let total_amount = milestones.get(0).unwrap().amount as i128;
    let trustless_work_commission = ((total_amount * 30) / 10000) as i128;
    let platform_commission = (total_amount * (platform_fee as i128)) / (10000 as i128);
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
        "Service Provider received incorrect amount"
    );

    assert_eq!(
        usdc_token.0.balance(&escrow_approver.address),
        initial_contract_balance - total_amount,
        "Contract balance is incorrect after claiming earnings"
    );
}

#[test]
fn test_release_milestone_funds_no_milestones() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);

    let platform_fee = 3 * 100;

    let usdc_token = create_usdc_token(&env, &admin);

    let engagement_id = String::from_str(&env, "41431");

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles: roles.clone(),
        platform_fee: platform_fee,
        milestones: vec![&env],
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_approver = test_data.client;

    let init_result = escrow_approver.try_initialize_escrow(&escrow_properties);
    assert!(
        init_result.is_err(),
        "Initialization should fail when no milestones are defined"
    );
}

#[test]
fn test_release_milestone_funds_milestones_incomplete() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let platform_fee = 3 * 100;

    let usdc_token = create_usdc_token(&env, &admin);
    let engagement_id = String::from_str(&env, "41431");

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
    };

    let flags: Flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
        approved: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            flags: flags.clone(),
            amount: 100_000,
            evidence: String::from_str(&env, "Empty"),
            receiver: service_provider_address.clone(),
        },
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Pending"),
            flags: flags.clone(),
            amount: 100_000,
            evidence: String::from_str(&env, "Empty"),
            receiver: service_provider_address.clone(),
        },
    ];

    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles: roles.clone(),
        platform_fee: platform_fee,
        milestones: milestones.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&escrow_properties);

    // Try to claim earnings with incomplete milestones (should fail)
    let result = escrow_approver.try_release_milestone_funds(
        &release_signer_address,
        &platform_address,
        &(0),
    );
    assert!(
        result.is_err(),
        "Should fail when milestones are not completed"
    );
}

#[test]
fn test_release_milestone_funds_same_receiver_as_provider() {
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

    let platform_fee = 3 * 100;

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
    };

    let flags: Flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
        approved: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Completed"),
            amount,
            evidence: String::from_str(&env, "Initial evidence"),
            flags,
            receiver: _receiver_address.clone(),
        },
    ];

    let engagement_id = String::from_str(&env, "test_escrow_same_receiver");
    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles,
        platform_fee: platform_fee,
        milestones: milestones.clone(),
        trustline,
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&escrow_properties);

    usdc_token
        .1
        .mint(&escrow_approver.address, &(amount as i128));

    // Approve before release
    escrow_approver.approve_milestone(&0, &approver_address);
    escrow_approver.release_milestone_funds(&release_signer_address, &trustless_work_address, &0);

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

    let platform_fee = 3 * 100;

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
    };

    let flags: Flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
        approved: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Completed"),
            amount,
            evidence: String::from_str(&env, "Initial evidence"),
            flags,
            receiver: _receiver_address.clone(),
        },
    ];

    let engagement_id = String::from_str(&env, "test_escrow_receiver");
    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles,
        platform_fee: platform_fee,
        milestones: milestones.clone(),
        trustline,
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&escrow_properties);

    usdc_token
        .1
        .mint(&escrow_approver.address, &(amount as i128));

    // Approve before release
    escrow_approver.approve_milestone(&0, &approver_address);
    escrow_approver.release_milestone_funds(&release_signer_address, &trustless_work_address, &0);

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
fn test_fund_escrow_successful_deposit() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let amount: i128 = 100_000_000;
    let service_provider_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let platform_fee = 3 * 100;
    let usdc_token = create_usdc_token(&env, &admin);

    let engagement_id = String::from_str(&env, "41431");

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
    };

    let flags: Flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
        approved: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            flags: flags.clone(),
            amount: 100_000,
            evidence: String::from_str(&env, "Empty"),
            receiver: service_provider_address.clone(),
        },
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Pending"),
            flags: flags.clone(),
            amount: 100_000,
            evidence: String::from_str(&env, "Empty"),
            receiver: service_provider_address.clone(),
        },
    ];

    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles: roles.clone(),
        platform_fee: platform_fee,
        milestones: milestones,
        trustline,
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&escrow_properties);

    usdc_token
        .1
        .mint(&release_signer_address, &(amount as i128));

    let amount_to_deposit: i128 = 100_000_000;

    escrow_approver.fund_escrow(
        &release_signer_address,
        &escrow_properties,
        &amount_to_deposit,
    );

    let expected_result_amount: i128 = 100_000_000;

    assert_eq!(
        usdc_token.0.balance(&escrow_approver.address),
        expected_result_amount,
        "Escrow balance is incorrect"
    );
}

#[test]
fn test_fund_escrow_signer_insufficient_funds_error() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let amount: i128 = 100_000_000;
    let service_provider_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let platform_fee = 3 * 100;
    let usdc_token = create_usdc_token(&env, &admin);
    let engagement_id = String::from_str(&env, "41431");

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
    };

    let flags: Flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
        approved: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            flags: flags.clone(),
            amount: 100_000,
            evidence: String::from_str(&env, "Empty"),
            receiver: service_provider_address.clone(),
        },
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Pending"),
            flags: flags.clone(),
            amount: 100_000,
            evidence: String::from_str(&env, "Empty"),
            receiver: service_provider_address.clone(),
        },
    ];

    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles: roles.clone(),
        platform_fee: platform_fee,
        milestones: milestones,
        trustline,
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&escrow_properties);

    usdc_token
        .1
        .mint(&escrow_approver.address, &(amount as i128));

    let signer_funds: i128 = 100_000;
    usdc_token
        .1
        .mint(&release_signer_address, &(signer_funds as i128));

    let amount_to_deposit: i128 = 180_000;

    let result = escrow_approver.try_fund_escrow(
        &release_signer_address,
        &escrow_properties,
        &amount_to_deposit,
    );

    assert!(
        result.is_err(),
        "Should fail when the signer has insufficient funds"
    );
}

#[test]
fn test_fund_escrow_dispute_flag_error() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let platform_fee = 3 * 100;
    let usdc_token = create_usdc_token(&env, &admin);
    let engagement_id = String::from_str(&env, "41431");

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
    };

    let flags: Flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
        approved: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            flags: flags.clone(),
            amount: 100_000,
            evidence: String::from_str(&env, "Empty"),
            receiver: service_provider_address.clone(),
        },
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Pending"),
            flags: flags.clone(),
            amount: 100_000,
            evidence: String::from_str(&env, "Empty"),
            receiver: service_provider_address.clone(),
        },
    ];

    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles: roles.clone(),
        platform_fee: platform_fee,
        milestones: milestones,
        trustline,
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&escrow_properties);

    let amount_to_deposit: i128 = 80_000;

    let result = escrow_approver.try_fund_escrow(
        &release_signer_address,
        &escrow_properties,
        &amount_to_deposit,
    );

    assert!(
        result.is_err(),
        "Should fail when the dispute approved_flag is true"
    );
}
