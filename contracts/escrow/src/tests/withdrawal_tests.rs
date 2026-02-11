#![cfg(test)]

use crate::storage::types::{Escrow, Flags, Milestone, MilestoneUpdate, Roles, Trustline};
use crate::tests::helpers::{create_escrow_contract, create_usdc_token};

use soroban_sdk::{testutils::Address as _, vec, Address, Env, Map, String};

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
        "Total withdrawn should equal the extra remaining amount"
    );
}
