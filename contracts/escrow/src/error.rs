use soroban_sdk::contracterror;

#[derive(Debug, Copy, Clone, PartialEq)]
#[contracterror]
pub enum ContractError {
    AmountCannotBeZero = 1,
    AmountsToBeTransferredShouldBePositive = 2,
    ApproverOrReceiverFundsLessThanZero = 3,
    CantReleaseAMilestoneInDispute = 4,
    DisputeResolverCannotDisputeTheMilestone = 5,
    DivisionError = 6,
    EmptyMilestoneStatus = 7,
    EscrowAlreadyInitialized = 8,
    EscrowAlreadyResolved = 9,
    EscrowBalanceMustBeZeroOnInitialization = 10,
    EscrowBalanceNotEnoughToSendEarnings = 11,
    EscrowHasFunds = 12,
    EscrowNotFound = 13,
    EscrowNotFullyProcessed = 14,
    EscrowPropertiesMismatch = 15,
    FlagsMustBeFalse = 16,
    IncompatibleEscrowWasmHash = 17,
    InsufficientApproverFundsForCommissions = 18,
    InsufficientEscrowFundsToMakeTheRefund = 19,
    InsufficientFundsForEscrowFunding = 20,
    InsufficientFundsForRefund = 21,
    InsufficientFundsForResolution = 22,
    InsufficientServiceProviderFundsForCommissions = 23,
    InvalidMileStoneIndex = 24,
    MilestoneAlreadyInDispute = 25,
    MilestoneAlreadyReleased = 26,
    MilestoneAlreadyResolved = 27,
    MilestoneApprovedCantChangeEscrowProperties = 28,
    MilestoneHasAlreadyBeenApproved = 29,
    MilestoneNotCompleted = 30,
    MilestoneNotFound = 31,
    MilestoneNotInDispute = 32,
    MilestoneOpenedForDisputeResolution = 33,
    MilestoneToApproveDoesNotExist = 34,
    MilestoneToUpdateDoesNotExist = 35,
    NoMileStoneDefined = 36,
    OnlyApproverChangeMilstoneFlag = 37,
    OnlyDisputeResolverCanExecuteThisFunction = 38,
    OnlyPlatformAddressExecuteThisFunction = 39,
    OnlyReleaseSignerCanReleaseEarnings = 40,
    OnlyServiceProviderChangeMilstoneStatus = 41,
    Overflow = 42,
    PlatformAddressCannotBeChanged = 43,
    PlatformFeeTooHigh = 44,
    TooManyEscrowsRequested = 45,
    TooManyMilestones = 46,
    TotalAmountCannotBeZero = 47,
    TotalDisputeFundsMustNotExceedTheMilestoneAmount = 48,
    UnauthorizedToChangeDisputeFlag = 49,
    Underflow = 50,
}

// impl fmt::Display for ContractError {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             ContractError::AmountCannotBeZero => write!(f, "Amount cannot be zero"),
//             ContractError::AmountsToBeTransferredShouldBePositive => {
//                 write!(
//                     f,
//                     "None of the amounts to be transferred should be less or equalP than 0."
//                 )
//             }
//             ContractError::ApproverOrReceiverFundsLessThanZero => {
//                 write!(
//                     f,
//                     "The funds of the approver or receiver must not be less or equal than 0."
//                 )
//             }
//             ContractError::CantReleaseAMilestoneInDispute => {
//                 write!(f, "You cannot launch a milestone in dispute")
//             }
//             ContractError::DisputeResolverCannotDisputeTheMilestone => {
//                 write!(
//                     f,
//                     "The dispute resolver cannot be the one to raise a dispute on a milestone."
//                 )
//             }
//             ContractError::DivisionError => write!(f, "This operation can cause Division error"),
//             ContractError::EmptyMilestoneStatus => {
//                 write!(f, "The milestone status cannot be empty")
//             }
//             ContractError::EscrowAlreadyInitialized => write!(f, "Escrow already initialized"),
//             ContractError::EscrowAlreadyResolved => {
//                 write!(f, "This escrow is already resolved.")
//             }
//             ContractError::EscrowBalanceMustBeZeroOnInitialization => {
//                 write!(f, "The escrow balance must be zero upon initialization.")
//             }
//             ContractError::EscrowBalanceNotEnoughToSendEarnings => write!(
//                 f,
//                 "The escrow balance must be equal to the amount of earnings defined for the escrow"
//             ),
//             ContractError::EscrowHasFunds => write!(f, "Escrow has funds"),
//             ContractError::EscrowNotFound => write!(f, "Escrow not found"),
//             ContractError::EscrowNotFullyProcessed => {
//                 write!(f, "All milestones must be released or dispute-resolved before withdrawing remaining funds")
//             }
//             ContractError::EscrowPropertiesMismatch => {
//                 write!(
//                     f,
//                     "The provided escrow properties do not match the stored escrow."
//                 )
//             }
//             ContractError::FlagsMustBeFalse => {
//                 write!(f, "All flags (approved, disputed, released) must be false in order to execute this function.")
//             }
//             ContractError::IncompatibleEscrowWasmHash => {
//                 write!(
//                     f,
//                     "The provided contract address is not an instance of this escrow contract."
//                 )
//             }
//             ContractError::InsufficientApproverFundsForCommissions => {
//                 write!(f, "Insufficient approver funds for commissions")
//             }
//             ContractError::InsufficientEscrowFundsToMakeTheRefund => {
//                 write!(
//                     f,
//                     "The escrow (contract) does not have sufficient funds to make the refund."
//                 )
//             }
//             ContractError::InsufficientFundsForEscrowFunding => {
//                 write!(f, "The signer has insufficient funds to fund the escrow.")
//             }
//             ContractError::InsufficientFundsForRefund => {
//                 write!(
//                     f,
//                     "Insufficient funds to refund the remaining funds from the escrow account."
//                 )
//             }
//             ContractError::InsufficientFundsForResolution => {
//                 write!(f, "Insufficient funds for resolution")
//             }
//             ContractError::InsufficientServiceProviderFundsForCommissions => {
//                 write!(f, "Insufficient Service Provider funds for commissions")
//             }
//             ContractError::InvalidMileStoneIndex => write!(f, "Invalid milestone index"),
//             ContractError::MilestoneAlreadyInDispute => write!(f, "Milestone already in dispute"),
//             ContractError::MilestoneAlreadyReleased => {
//                 write!(f, "This milestone is already released")
//             }
//             ContractError::MilestoneAlreadyResolved => {
//                 write!(f, "This milestone is already resolved")
//             }
//             ContractError::MilestoneApprovedCantChangeEscrowProperties => {
//                 write!(
//                     f,
//                     "You cannot change the properties of an escrow after one of the milestones has been marked as approved"
//                 )
//             }
//             ContractError::MilestoneHasAlreadyBeenApproved => {
//                 write!(
//                     f,
//                     "You cannot approve a milestone that has already been approved previously"
//                 )
//             }
//             ContractError::MilestoneNotCompleted => {
//                 write!(f, "The milestone must be completed to release funds")
//             }
//             ContractError::MilestoneNotFound => write!(f, "Milestone not found"),
//             ContractError::MilestoneNotInDispute => write!(f, "Milestone not in dispute"),
//             ContractError::MilestoneOpenedForDisputeResolution => {
//                 write!(f, "Milestone has been opened for dispute resolution")
//             }
//             ContractError::MilestoneToApproveDoesNotExist => {
//                 write!(f, "One of the selected milestones to approve does not exist")
//             }
//             ContractError::MilestoneToUpdateDoesNotExist => {
//                 write!(f, "One of the selected milestones to update does not exist in the milestones list")
//             }
//             ContractError::NoMileStoneDefined => write!(f, "Escrow initialized without milestone"),
//             ContractError::OnlyApproverChangeMilstoneFlag => {
//                 write!(f, "Only the approver can change milestone flag")
//             }
//             ContractError::OnlyDisputeResolverCanExecuteThisFunction => {
//                 write!(f, "Only the dispute resolver can execute this function")
//             }
//             ContractError::OnlyPlatformAddressExecuteThisFunction => write!(
//                 f,
//                 "Only the platform address should be able to execute this function"
//             ),
//             ContractError::OnlyReleaseSignerCanReleaseEarnings => {
//                 write!(f, "Only the release signer can release the escrow funds")
//             }
//             ContractError::OnlyServiceProviderChangeMilstoneStatus => {
//                 write!(f, "Only the service provider can change milestone status")
//             }
//             ContractError::Overflow => write!(f, "This operation can cause an Overflow"),
//             ContractError::PlatformAddressCannotBeChanged => {
//                 write!(f, "The platform address of the escrow cannot be changed.")
//             }
//             ContractError::PlatformFeeTooHigh => {
//                 write!(f, "The platform fee cannot exceed 99%")
//             }
//             ContractError::TooManyEscrowsRequested => {
//                 write!(f, "You have requested too many escrows")
//             }
//             ContractError::TooManyMilestones => {
//                 write!(f, "Cannot define more than 50 milestones in an escrow")
//             }
//             ContractError::TotalAmountCannotBeZero => {
//                 write!(f, "The total amount to be transferred cannot be zero.")
//             }
//             ContractError::TotalDisputeFundsMustNotExceedTheMilestoneAmount => {
//                 write!(f, "The total funds to resolve the dispute must not exceed the amount defined for this milestone.")
//             }
//             ContractError::UnauthorizedToChangeDisputeFlag => {
//                 write!(f, "You are not authorized to change the dispute flag")
//             }
//             ContractError::Underflow => write!(f, "This operation can cause an Underflow"),
//         }
//     }
// }
