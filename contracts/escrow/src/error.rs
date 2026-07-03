use soroban_sdk::contracterror;

#[derive(Debug, Copy, Clone, PartialEq)]
#[contracterror]
pub enum EscrowError {
    EscrowAlreadyInitialized = 1,
    EscrowNotFound = 2,
    EscrowAlreadyReleased = 3,
    EscrowAlreadyResolved = 4,
    EscrowNotInDispute = 5,
    EscrowOpenedForDisputeResolution = 6,
    EscrowNotCompleted = 7,
    EscrowBalanceNotEnoughToSendEarnings = 8,
    EscrowPropertiesMismatch = 9,
    FlagsMustBeFalse = 10,
    AmountCannotBeZero = 11,
    PlatformFeeTooHigh = 12,
    InsufficientFundsForEscrowFunding = 13,
    TooManyEscrowsRequested = 14,
    InsufficientFundsForResolution = 15,
    DistributionsMustEqualEscrowBalance = 16,
    AmountsToBeTransferredShouldBePositive = 17,
    TotalAmountCannotBeZero = 18,
    TooManyDistributions = 19,
    EscrowNotFullyProcessed = 20,
    Overflow = 21,
    Underflow = 22,
    DivisionError = 23,
    OnlyReleaseSignerCanReleaseEarnings = 24,
    OnlyDisputeResolverCanExecuteThisFunction = 25,
    UnauthorizedToChangeDisputeFlag = 26,
    DisputeResolverCannotDisputeTheEscrow = 27,
    OnlyAdminAddressExecuteThisFunction = 28,
    AdminAddressCannotBeChanged = 29,
    AdminAddressOverlapsWithOtherRole = 30,
    ApproversListEmpty = 31,
    ServiceProvidersListEmpty = 32,
    ReleaseSignersListEmpty = 33,
    DisputeResolversListEmpty = 34,
    NoMilestoneDefined = 35,
    TooManyMilestones = 36,
    TargetCannotBeZero = 37,
    PlatformAddressCannotBeChanged = 38,
    ReleaseMilestonesEmpty = 39,
    MilestoneAlreadyReleased = 40,
    InvalidMilestoneIndex = 41,
    BatchMilestoneDisputeEmpty = 42,
    MilestoneAlreadyDisputed = 43,
    RoleLimitExceeded = 44,
    DuplicateAddressInRole = 45,
    DisputeResolverOverlapsWithOtherRole = 46,
    MilestoneUpdateNotAllowedWithFunds = 47,
    TargetExceedsApprovers = 48,
    StringTooLong = 49,
    SignerMustBeApproverAndReleaseSigner = 50,
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[contracterror]
pub enum CctpError {
    InvalidDestinationDomain = 1,
    InvalidRecipient = 2,
    OnlyReceiverCanSetDestination = 3,
    DestinationNotSet = 4,
    MilestoneNotFound = 5,
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[contracterror]
pub enum ReleaseError {
    EscrowAlreadyResolved = 1,
    OnlyReleaseSignerCanReleaseEarnings = 2,
    EscrowOpenedForDisputeResolution = 3,
    ReleaseMilestonesEmpty = 4,
    DuplicateMilestoneIndex = 5,
    InvalidMilestoneIndex = 6,
    EscrowNotCompleted = 7,
    MilestoneAlreadyReleased = 8,
    EscrowBalanceNotEnoughToSendEarnings = 9,
    Overflow = 10,
    Underflow = 11,
    DivisionError = 12,
    EscrowNotFound = 13,
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
                EscrowError::MilestoneAlreadyReleased
            }
            MilestoneError::EscrowNotFound => EscrowError::EscrowNotFound,
            _ => EscrowError::EscrowNotCompleted,
        }
    }
}

impl From<ReleaseError> for EscrowError {
    fn from(err: ReleaseError) -> EscrowError {
        match err {
            ReleaseError::EscrowNotFound => EscrowError::EscrowNotFound,
            ReleaseError::EscrowOpenedForDisputeResolution => {
                EscrowError::EscrowOpenedForDisputeResolution
            }
            ReleaseError::EscrowNotCompleted => EscrowError::EscrowNotCompleted,
            ReleaseError::MilestoneAlreadyReleased => EscrowError::MilestoneAlreadyReleased,
            ReleaseError::EscrowBalanceNotEnoughToSendEarnings => {
                EscrowError::EscrowBalanceNotEnoughToSendEarnings
            }
            ReleaseError::Overflow => EscrowError::Overflow,
            ReleaseError::Underflow => EscrowError::Underflow,
            ReleaseError::DivisionError => EscrowError::DivisionError,
            _ => EscrowError::EscrowNotCompleted,
        }
    }
}
