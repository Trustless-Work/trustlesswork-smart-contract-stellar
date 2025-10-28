use core::fmt;
use soroban_sdk::contracterror;

#[derive(Debug, Copy, Clone, PartialEq)]
#[contracterror]
pub enum ContractError {
    AmountCannotBeZero = 1,
    EscrowAlreadyInitialized = 2,
    EscrowNotFound = 3,
    OnlyReleaseSignerCanReleaseEarnings = 4,
    EscrowNotCompleted = 5,
    EscrowBalanceNotEnoughToSendEarnings = 6,
    OnlyPlatformAddressExecuteThisFunction = 7,
    OnlyServiceProviderChangeMilstoneStatus = 8,
    NoMilestoneDefined = 9,
    InvalidMileStoneIndex = 10,
    OnlyApproverChangeMilstoneFlag = 11,
    OnlyDisputeResolverCanExecuteThisFunction = 12,
    EscrowAlreadyInDispute = 13,
    EscrowNotInDispute = 14,
    InsufficientFundsForResolution = 15,
    EscrowOpenedForDisputeResolution = 16,
    Overflow = 17,
    Underflow = 18,
    DivisionError = 19,
    InsufficientApproverFundsForCommissions = 20,
    InsufficientServiceProviderFundsForCommissions = 21,
    MilestoneApprovedCantChangeEscrowProperties = 22,
    EscrowHasFunds = 23,
    EscrowAlreadyResolved = 24,
    TooManyEscrowsRequested = 25,
    UnauthorizedToChangeDisputeFlag = 26,
    TooManyMilestones = 27,
    ReceiverAndApproverFundsNotEqual = 28,
    AmountsToBeTransferredShouldBePositive = 38,
    DistributionsMustEqualEscrowBalance = 39,
    MilestoneHasAlreadyBeenApproved = 29,
    EmptyMilestoneStatus = 30,
    PlatformFeeTooHigh = 31,
    FlagsMustBeFalse = 32,
    EscrowPropertiesMismatch = 33,
    ApproverOrReceiverFundsLessThanZero = 34,
    EscrowAlreadyReleased = 35,
    IncompatibleEscrowWasmHash = 36,
    PlatformAddressCannotBeChanged = 37,
    DisputeResolverCannotDisputeTheEscrow = 40,
    TotalAmountCannotBeZero = 41,
}

impl fmt::Display for ContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContractError::AmountCannotBeZero => {
                write!(f, "Amount cannot be equal to or less than zero")
            }
            ContractError::EscrowAlreadyInitialized => write!(f, "Escrow already initialized"),
            ContractError::EscrowNotFound => write!(f, "Escrow not found"),
            ContractError::OnlyReleaseSignerCanReleaseEarnings => {
                write!(f, "Only the release signer can release the escrow earnings")
            }
            ContractError::EscrowNotCompleted => {
                write!(f, "The escrow must be completed to release earnings")
            }
            ContractError::EscrowBalanceNotEnoughToSendEarnings => write!(
                f,
                "The escrow balance must be equal to the amount of earnings defined for the escrow"
            ),
            ContractError::OnlyPlatformAddressExecuteThisFunction => write!(
                f,
                "Only the platform address should be able to execute this function"
            ),
            ContractError::OnlyServiceProviderChangeMilstoneStatus => {
                write!(f, "Only the service provider can change milestone status")
            }
            ContractError::NoMilestoneDefined => write!(f, "Escrow initialized without milestone"),
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
            ContractError::EscrowOpenedForDisputeResolution => {
                write!(f, "Escrow has been opened for dispute resolution")
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
            ContractError::EscrowAlreadyResolved => write!(f, "This escrow is already resolved"),
            ContractError::TooManyEscrowsRequested => {
                write!(f, "You have requested too many escrows")
            }
            ContractError::UnauthorizedToChangeDisputeFlag => {
                write!(f, "You are not authorized to change the dispute flag")
            }
            ContractError::TooManyMilestones => {
                write!(f, "Cannot define more than 50 milestones in an escrow")
            }
            ContractError::ReceiverAndApproverFundsNotEqual => {
                write!(
                    f,
                    "The approver's and receiver's funds must equal the current escrow balance."
                )
            }
            ContractError::AmountsToBeTransferredShouldBePositive => {
                write!(
                    f,
                    "None of the amounts to be transferred should be less or equal than 0."
                )
            }
            ContractError::DistributionsMustEqualEscrowBalance => {
                write!(f, "The sum of distributions must equal the current escrow balance when resolving an escrow dispute.")
            }
            ContractError::MilestoneHasAlreadyBeenApproved => {
                write!(
                    f,
                    "You cannot approve a milestone that has already been approved previously"
                )
            }
            ContractError::EmptyMilestoneStatus => {
                write!(f, "The milestone status cannot be empty")
            }
            ContractError::PlatformFeeTooHigh => {
                write!(f, "The platform fee cannot exceed 99%")
            }
            ContractError::FlagsMustBeFalse => {
                write!(f, "All flags (approved, disputed, released) must be false in order to execute this function.")
            }
            ContractError::EscrowPropertiesMismatch => {
                write!(
                    f,
                    "The provided escrow properties do not match the stored escrow."
                )
            }
            ContractError::ApproverOrReceiverFundsLessThanZero => {
                write!(
                    f,
                    "The funds of the approver or receiver must not be less or equal than 0."
                )
            }
            ContractError::EscrowAlreadyReleased => {
                write!(f, "The escrow funds have been released.")
            }
            ContractError::IncompatibleEscrowWasmHash => {
                write!(
                    f,
                    "The provided contract address is not an instance of this escrow contract."
                )
            }
            ContractError::PlatformAddressCannotBeChanged => {
                write!(f, "The platform address of the escrow cannot be changed.")
            }
            ContractError::DisputeResolverCannotDisputeTheEscrow => {
                write!(f, "The dispute resolver cannot dispute the escrow.")
            }
            ContractError::TotalAmountCannotBeZero => {
                write!(f, "The total amount to be distributed cannot be equal to zero.")
            }
        }
    }
}
