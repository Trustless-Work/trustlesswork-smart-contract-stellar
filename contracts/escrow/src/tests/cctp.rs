extern crate std;

use crate::error::CctpError;
use crate::modules::cctp::constants::CCTP_TOKEN_MESSENGER_STRKEY;
use crate::modules::cctp::release::release_receiver_amount_via_cctp_with_messenger;
use crate::storage::types::{
    Dispute, Escrow, Milestone, MilestoneApprovals, Roles, Trustline,
};
use soroban_sdk::{
    contract, contractimpl, testutils::Address as _, token, vec, Address, BytesN, Env, String,
};
use token::Client as TokenClient;

use super::helpers::{create_escrow_contract, create_usdc_token};

/// Mock standing in for Circle's TokenMessengerMinter: requires the caller's
/// auth and pulls the approved USDC from the caller via the allowance.
#[contract]
pub struct MockTokenMessenger;

#[contractimpl]
impl MockTokenMessenger {
    #[allow(clippy::too_many_arguments)]
    pub fn deposit_for_burn(
        e: Env,
        caller: Address,
        amount: i128,
        _destination_domain: u32,
        _mint_recipient: BytesN<32>,
        burn_token: Address,
        _destination_caller: BytesN<32>,
        _max_fee: i128,
        _min_finality_threshold: u32,
    ) {
        caller.require_auth();
        let self_addr = e.current_contract_address();
        TokenClient::new(&e, &burn_token).transfer_from(&self_addr, &caller, &self_addr, &amount);
    }
}

fn evm_recipient(e: &Env, byte: u8) -> BytesN<32> {
    let mut bytes = [0u8; 32];
    bytes[31] = byte;
    BytesN::from_array(e, &bytes)
}

fn milestone(env: &Env, amount: i128, receiver: &Address) -> Milestone {
    Milestone {
        description: String::from_str(env, "M"),
        status: String::from_str(env, "Completed"),
        evidence: String::from_str(env, "Done"),
        approvals: MilestoneApprovals {
            target: 1,
            approval_count: 0,
            approved_by: vec![env],
        },
        amount,
        dispute: Dispute {
            is_disputed: false,
            reason: String::from_str(env, ""),
            resolved: false,
        },
        released: false,
        receiver: receiver.clone(),
    }
}

struct EscrowFixture {
    escrow: Escrow,
    admin: Address,
    approver: Address,
    release_signer: Address,
    platform: Address,
    receiver0: Address,
    receiver1: Address,
}

/// Two milestones with DIFFERENT receivers, to prove per-milestone routing.
fn base_escrow(env: &Env, usdc: &Address, amount_each: i128, platform_fee: u32) -> EscrowFixture {
    let admin = Address::generate(env);
    let approver = Address::generate(env);
    let release_signer = Address::generate(env);
    let platform = Address::generate(env);
    let receiver0 = Address::generate(env);
    let receiver1 = Address::generate(env);

    let roles = Roles {
        approvers: vec![env, approver.clone()],
        service_providers: vec![env, Address::generate(env)],
        platform: platform.clone(),
        release_signers: vec![env, release_signer.clone()],
        dispute_resolvers: vec![env, Address::generate(env)],
        admin: admin.clone(),
        observers: vec![env],
    };

    let escrow = Escrow {
        engagement_id: String::from_str(env, "cctp_multi"),
        title: String::from_str(env, "CCTP Multi"),
        description: String::from_str(env, "Per-milestone cross-chain"),
        roles,
        platform_fee,
        milestones: vec![
            env,
            milestone(env, amount_each, &receiver0),
            milestone(env, amount_each, &receiver1),
        ],
        trustline: Trustline {
            address: usdc.clone(),
        },
        receiver_memo: 0,
    };

    EscrowFixture {
        escrow,
        admin,
        approver,
        release_signer,
        platform,
        receiver0,
        receiver1,
    }
}

fn net_of(amount: i128, platform_fee: u32) -> (i128, i128, i128) {
    let tw = amount * 30 / 10_000;
    let plat = amount * platform_fee as i128 / 10_000;
    (tw, plat, amount - tw - plat)
}

#[test]
fn release_routes_milestone_to_cctp_when_its_receiver_registered() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let usdc = create_usdc_token(&env, &admin);

    let messenger = Address::from_str(&env, CCTP_TOKEN_MESSENGER_STRKEY);
    env.register_at(&messenger, MockTokenMessenger, ());

    let each: i128 = 50_000_000;
    let platform_fee: u32 = 500;
    let f = base_escrow(&env, &usdc.0.address, each, platform_fee);
    let tw_address = Address::generate(&env);

    let client = create_escrow_contract(&env, &f.admin).client;
    client.initialize_escrow(&f.escrow);
    usdc.1.mint(&client.address, &(each * 2));

    // Only milestone 0's receiver registers a cross-chain destination.
    client.set_cross_chain_destination(&f.receiver0, &0u32, &6, &evm_recipient(&env, 0xAB));

    client.approve_milestones(&vec![&env, 0u32], &f.approver);
    client.approve_milestones(&vec![&env, 1u32], &f.approver);
    client.release_funds(&f.release_signer, &tw_address, &vec![&env, 0u32, 1u32]);

    let (_tw, _plat, net) = net_of(each, platform_fee);

    // Milestone 0 -> burned via the messenger; milestone 1 -> Stellar receiver.
    assert_eq!(usdc.0.balance(&messenger), net);
    assert_eq!(usdc.0.balance(&f.receiver0), 0);
    assert_eq!(usdc.0.balance(&f.receiver1), net);
    assert_eq!(usdc.0.balance(&client.address), 0);
}

#[test]
fn release_stays_on_stellar_without_registered_destination() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let usdc = create_usdc_token(&env, &admin);

    let each: i128 = 50_000_000;
    let platform_fee: u32 = 500;
    let f = base_escrow(&env, &usdc.0.address, each, platform_fee);
    let tw_address = Address::generate(&env);

    let client = create_escrow_contract(&env, &f.admin).client;
    client.initialize_escrow(&f.escrow);
    usdc.1.mint(&client.address, &(each * 2));

    client.approve_milestones(&vec![&env, 0u32], &f.approver);
    client.release_funds(&f.release_signer, &tw_address, &vec![&env, 0u32]);

    let (_tw, _plat, net) = net_of(each, platform_fee);
    assert_eq!(usdc.0.balance(&f.receiver0), net);
}

#[test]
fn only_milestone_receiver_can_set_destination() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let usdc = create_usdc_token(&env, &admin);

    let f = base_escrow(&env, &usdc.0.address, 50_000_000, 500);
    let client = create_escrow_contract(&env, &f.admin).client;
    client.initialize_escrow(&f.escrow);

    // receiver1 tries to set the destination for milestone 0 (not theirs).
    let res = client.try_set_cross_chain_destination(
        &f.receiver1,
        &0u32,
        &6,
        &evm_recipient(&env, 0xAB),
    );
    assert_eq!(
        res,
        Err(Ok(CctpError::OnlyReceiverCanSetDestination))
    );
}

#[test]
fn set_destination_rejects_invalid_milestone_index() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let usdc = create_usdc_token(&env, &admin);

    let f = base_escrow(&env, &usdc.0.address, 50_000_000, 500);
    let client = create_escrow_contract(&env, &f.admin).client;
    client.initialize_escrow(&f.escrow);

    let res = client.try_set_cross_chain_destination(
        &f.receiver0,
        &99u32,
        &6,
        &evm_recipient(&env, 0xAB),
    );
    assert_eq!(res, Err(Ok(CctpError::MilestoneNotFound)));
}

#[test]
fn set_destination_rejects_invalid_domain_and_zero_recipient() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let usdc = create_usdc_token(&env, &admin);

    let f = base_escrow(&env, &usdc.0.address, 50_000_000, 500);
    let client = create_escrow_contract(&env, &f.admin).client;
    client.initialize_escrow(&f.escrow);

    let bad_domain =
        client.try_set_cross_chain_destination(&f.receiver0, &0u32, &999, &evm_recipient(&env, 1));
    assert_eq!(bad_domain, Err(Ok(CctpError::InvalidDestinationDomain)));

    let zero_recipient = client.try_set_cross_chain_destination(
        &f.receiver0,
        &0u32,
        &6,
        &BytesN::from_array(&env, &[0u8; 32]),
    );
    assert_eq!(zero_recipient, Err(Ok(CctpError::InvalidRecipient)));
}

#[test]
fn receiver_can_clear_destination_to_revert_to_stellar() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let usdc = create_usdc_token(&env, &admin);

    let each: i128 = 50_000_000;
    let platform_fee: u32 = 500;
    let f = base_escrow(&env, &usdc.0.address, each, platform_fee);
    let tw_address = Address::generate(&env);

    let client = create_escrow_contract(&env, &f.admin).client;
    client.initialize_escrow(&f.escrow);
    usdc.1.mint(&client.address, &(each * 2));

    client.set_cross_chain_destination(&f.receiver0, &0u32, &6, &evm_recipient(&env, 0xAB));
    client.clear_cross_chain_destination(&f.receiver0, &0u32);

    client.approve_milestones(&vec![&env, 0u32], &f.approver);
    client.release_funds(&f.release_signer, &tw_address, &vec![&env, 0u32]);

    let (_tw, _plat, net) = net_of(each, platform_fee);
    assert_eq!(usdc.0.balance(&f.receiver0), net);
}

#[test]
fn helper_sends_seventh_decimal_remainder_to_stellar() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let usdc = create_usdc_token(&env, &admin);

    let escrow_contract = env.register(
        crate::contract::EscrowContract,
        (&admin, &BytesN::from_array(&env, &[0u8; 32])),
    );
    let amount: i128 = 1_0000003;
    usdc.1.mint(&escrow_contract, &amount);

    let mock_messenger = env.register(MockTokenMessenger, ());
    let stellar_receiver = Address::generate(&env);

    release_receiver_amount_via_cctp_with_messenger(
        &env,
        &usdc.0,
        &escrow_contract,
        &mock_messenger,
        &usdc.0.address,
        amount,
        6,
        &evm_recipient(&env, 0xAB),
        &stellar_receiver,
    );

    assert_eq!(usdc.0.balance(&mock_messenger), 1_0000000);
    assert_eq!(usdc.0.balance(&stellar_receiver), 3);
    assert_eq!(usdc.0.balance(&escrow_contract), 0);
}
