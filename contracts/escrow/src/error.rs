use soroban_sdk::contracterror;

#[derive(Debug, Copy, Clone, PartialEq)]
#[contracterror]
pub enum EscrowError {
    EscrowAlreadyInitialized = 1,
    EscrowNotFound = 2,
    EscrowAlreadyReleased = 3,
    EscrowAlreadyResolved = 4,
    EscrowAlreadyInDispute = 5,
    EscrowNotInDispute = 6,
    EscrowOpenedForDisputeResolution = 7,
    EscrowNotCompleted = 8,
    EscrowBalanceNotEnoughToSendEarnings = 9,
    EscrowPropertiesMismatch = 10,
    FlagsMustBeFalse = 11,
    AmountCannotBeZero = 12,
    PlatformFeeTooHigh = 13,
    InsufficientFundsForEscrowFunding = 14,
    TooManyEscrowsRequested = 15,
    InsufficientFundsForResolution = 16,
    DistributionsMustEqualEscrowBalance = 17,
    AmountsToBeTransferredShouldBePositive = 18,
    TotalAmountCannotBeZero = 19,
    TooManyDistributions = 20,
    EscrowNotFullyProcessed = 21,
    Overflow = 22,
    Underflow = 23,
    DivisionError = 24,
    OnlyReleaseSignerCanReleaseEarnings = 25,
    OnlyDisputeResolverCanExecuteThisFunction = 26,
    UnauthorizedToChangeDisputeFlag = 27,
    DisputeResolverCannotDisputeTheEscrow = 28,
    OnlyAdminAddressExecuteThisFunction = 29,
    AdminAddressCannotBeChanged = 30,
    AdminAddressOverlapsWithOtherRole = 31,
    ApproversListEmpty = 32,
    ServiceProvidersListEmpty = 33,
    ReleaseSignersListEmpty = 34,
    DisputeResolversListEmpty = 35,
    NoMilestoneDefined = 36,
    TooManyMilestones = 37,
    TargetCannotBeZero = 38,
    PlatformAddressCannotBeChanged = 39,
    InvalidMilestoneIndex = 40,
    RoleLimitExceeded = 41,
    DuplicateAddressInRole = 42,
    DisputeResolverOverlapsWithOtherRole = 43,
    MilestoneUpdateNotAllowedWithFunds = 44,
    TargetExceedsApprovers = 45,
    StringTooLong = 46,
    Reentrancy = 47,
    SignerMustBeApproverAndReleaseSigner = 48,
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[contracterror]
pub enum CctpError {
    InvalidDestinationDomain = 1,
    InvalidRecipient = 2,
    OnlyReceiverCanSetDestination = 3,
    DestinationNotSet = 4,
    /// `max_fee` must be non-negative and can't exceed a sane share of the
    /// route's amount — defense-in-depth against a bogus/compromised max_fee
    /// (the API computes this from a live Circle quote, but the contract
    /// itself doesn't trust that; anyone with the receiver's key can call
    /// this entrypoint directly, bypassing the API).
    MaxFeeExceedsCap = 5,
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[contracterror]
pub enum MilestoneError {
    NoMilestoneDefined = 1,
    InvalidMilestoneIndex = 2,
    MilestoneHasAlreadyBeenApproved = 3,
    ApproverAlreadyApprovedMilestone = 4,
    EmptyMilestoneStatus = 5,
    MilestoneToApproveDoesNotExist = 6,
    MilestoneToUpdateDoesNotExist = 7,
    BatchMilestoneUpdateEmpty = 8,
    BatchMilestoneApproveEmpty = 9,
    OnlyServiceProviderCanChangeMilestoneStatus = 10,
    UnauthorizedApprover = 11,
    EscrowNotFound = 12,
    DuplicateMilestoneIndex = 13,
    StringTooLong = 14,
    BatchTooLarge = 15,
}

impl From<MilestoneError> for EscrowError {
    fn from(err: MilestoneError) -> EscrowError {
        match err {
            MilestoneError::BatchMilestoneApproveEmpty => EscrowError::NoMilestoneDefined,
            MilestoneError::InvalidMilestoneIndex
            | MilestoneError::MilestoneToApproveDoesNotExist => EscrowError::InvalidMilestoneIndex,
            MilestoneError::DuplicateMilestoneIndex => EscrowError::InvalidMilestoneIndex,
            MilestoneError::MilestoneHasAlreadyBeenApproved
            | MilestoneError::ApproverAlreadyApprovedMilestone => {
                EscrowError::EscrowAlreadyReleased
            }
            MilestoneError::EscrowNotFound => EscrowError::EscrowNotFound,
            _ => EscrowError::EscrowNotCompleted,
        }
    }
}
