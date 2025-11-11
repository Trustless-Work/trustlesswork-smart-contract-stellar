use core::fmt;
use soroban_sdk::contracterror;

#[derive(Debug, Copy, Clone, PartialEq)]
#[contracterror]
pub enum ContractError {
    EscrowNotFunded = 1,
    AmountCannotBeZero = 2,
    EscrowAlreadyInitialized = 3,
    OnlySignerCanFundEscrow = 4,
    EscrowAlreadyFunded = 5,
    EscrowFullyFunded = 6,
    SignerInsufficientFunds = 7,
    NotEnoughAllowance = 8,
    EscrowAlreadyCompleted = 9,
    SignerInsufficientFundsToComplete = 10,
    OnlySignerCanRequestRefund = 11,
    NoFundsToRefund = 12,
    ContractHasInsufficientBalance = 13,
    EscrowNotFound = 14,
    OnlyReleaseSignerCanReleaseEarnings = 15,
    EscrowNotCompleted = 16,
    EscrowBalanceNotEnoughToSendEarnings = 17,
    ContractInsufficientFunds = 18,
    OnlyPlatformAddressExecuteThisFunction = 19,
    EscrowNotInitialized = 20,
    OnlyServiceProviderChangeMilstoneStatus = 21,
    NoMileStoneDefined = 22,
    InvalidMileStoneIndex = 23,
    OnlyApproverChangeMilstoneFlag = 24,
    OnlyDisputeResolverCanExecuteThisFunction = 25,
    EscrowAlreadyInDispute = 26,
    EscrowNotInDispute = 27,
    InsufficientFundsForResolution = 28,
    InvalidState = 29,
    EscrowOpenedForDisputeResolution = 30,
    AmountToDepositGreatherThanEscrowAmount = 31,
    Overflow = 32,
    Underflow = 33,
    DivisionError = 34,
    AdminNotFound = 35,
    InsufficientApproverFundsForCommissions = 36,
    InsufficientServiceProviderFundsForCommissions = 37,
    MilestoneApprovedCantChangeEscrowProperties = 38,
    EscrowHasFunds = 39,
    EscrowAlreadyResolved = 40,
    TooManyEscrowsRequested = 41,
    UnauthorizedToChangeDisputeFlag = 42,
    ArgumentConversionFailed = 43,
    TooManyMilestones = 44,
    ReceiverAndApproverFundsNotEqual = 45,
}

impl fmt::Display for ContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContractError::EscrowNotFunded => write!(f, "Escrow not funded"),
            ContractError::AmountCannotBeZero => write!(f, "Amount cannot be zero"),
            ContractError::EscrowAlreadyInitialized => write!(f, "Escrow already initialized"),
            ContractError::OnlySignerCanFundEscrow => {
                write!(f, "Only the signer can fund the escrow")
            }
            ContractError::EscrowAlreadyFunded => write!(f, "Escrow already funded"),
            ContractError::EscrowFullyFunded => write!(f, "This escrow is already fully funded"),
            ContractError::SignerInsufficientFunds => {
                write!(f, "The signer does not have sufficient funds")
            }
            ContractError::NotEnoughAllowance => {
                write!(f, "Not enough allowance to fund this escrow")
            }
            ContractError::EscrowAlreadyCompleted => write!(f, "Escrow already completed"),
            ContractError::SignerInsufficientFundsToComplete => write!(
                f,
                "The signer does not have sufficient funds to complete this escrow"
            ),
            ContractError::OnlySignerCanRequestRefund => {
                write!(f, "Only the signer can request a refund")
            }
            ContractError::NoFundsToRefund => write!(f, "No funds available to refund"),
            ContractError::ContractHasInsufficientBalance => {
                write!(f, "The contract has no balance to repay")
            }
            ContractError::EscrowNotFound => write!(f, "Escrow not found"),
            ContractError::OnlyReleaseSignerCanReleaseEarnings => write!(
                f,
                "Only the release signer can release the escrow earnings"
            ),
            ContractError::EscrowNotCompleted => {
                write!(f, "The escrow must be completed to release earnings")
            }
            ContractError::EscrowBalanceNotEnoughToSendEarnings => write!(
                f,
                "The escrow balance must be equal to the amount of earnings defined for the escrow"
            ),
            ContractError::ContractInsufficientFunds => {
                write!(f, "The contract does not have sufficient funds")
            }
            ContractError::OnlyPlatformAddressExecuteThisFunction => write!(
                f,
                "Only the platform address should be able to execute this function"
            ),
            ContractError::EscrowNotInitialized => write!(f, "Escrow not initialized"),
            ContractError::OnlyServiceProviderChangeMilstoneStatus => {
                write!(f, "Only the service provider can change milestone status")
            }
            ContractError::NoMileStoneDefined => write!(f, "Escrow initialized without milestone"),
            ContractError::InvalidMileStoneIndex => write!(f, "Invalid milestone index"),
            ContractError::OnlyApproverChangeMilstoneFlag => {
                write!(f, "Only the approver can change milestone flag")
            }
            ContractError::OnlyDisputeResolverCanExecuteThisFunction => {
                write!(f, "Only the dispute resolver can execute this function")
            }
            ContractError::EscrowAlreadyInDispute => write!(f, "Escrow already in dispute"),
            ContractError::EscrowNotInDispute => write!(f, "Escrow not in dispute"),
            ContractError::InsufficientFundsForResolution => {
                write!(f, "Insufficient funds for resolution")
            }
            ContractError::InvalidState => write!(f, "Invalid state"),
            ContractError::EscrowOpenedForDisputeResolution => {
                write!(f, "Escrow has been opened for dispute resolution")
            }
            ContractError::AmountToDepositGreatherThanEscrowAmount => {
                write!(f, "Amount to deposit is greater than the escrow amount")
            }
            ContractError::InsufficientApproverFundsForCommissions => {
                write!(f, "Insufficient approver funds for commissions")
            }
            ContractError::InsufficientServiceProviderFundsForCommissions => {
                write!(f, "Insufficient Service Provider funds for commissions")
            }
            ContractError::MilestoneApprovedCantChangeEscrowProperties => {
                write!(
                    f,
                    "You can't change the escrow properties after the milestone is approved"
                )
            }
            ContractError::EscrowHasFunds => write!(f, "Escrow has funds"),
            ContractError::Overflow => write!(f, "This operation can cause an Overflow"),
            ContractError::Underflow => write!(f, "This operation can cause an Underflow"),
            ContractError::DivisionError => write!(f, "This operation can cause Division error"),
            ContractError::AdminNotFound => write!(f, "Admin not found!"),
            ContractError::EscrowAlreadyResolved => write!(f, "This escrow is already resolved"),
            ContractError::TooManyEscrowsRequested => {
                write!(f, "You have requested too many escrows")
            }
            ContractError::UnauthorizedToChangeDisputeFlag => {
                write!(f, "You are not authorized to change the dispute flag")
            }
            ContractError::ArgumentConversionFailed => {
                write!(f, "Argument conversion failed")
            }
            ContractError::TooManyMilestones => {
                write!(f, "Cannot define more than 10 milestones in an escrow")
            }
            ContractError::ReceiverAndApproverFundsNotEqual => {
                write!(f, "The approver's and receiver's funds must equal the current escrow balance.")
            }
        }
    }
}
