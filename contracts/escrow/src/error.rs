use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum EscrowError {
    UnknownError = 0,
    AccountNotFound = 1,
    AlreadyResolved = 2,
    AlreadyTerminated = 3,
    InsufficientFunds = 4,
    InvalidClaim = 5,
    InvalidDuration = 6,
    InvalidEscrowId = 7,
    FlagsMustBeFalse = 10,
    InvalidSigner = 11,
    MissingClaimData = 12,
    InvalidReleaseType = 13,
    NotInitiated = 14,
    Reentrancy = 15,
    AlreadyDisputed = 16,
    DisputeNotInProgress = 17,
    InvalidDisputeOutcome = 18,
    TimeoutNotElapsed = 19,
    InvalidArbitrator = 20,
    MultipleReleasesNotSupported = 21,
    ReleaseAlreadyClaimed = 22,
    InvalidReleaseAmount = 23,
    InvalidTimestamp = 24,
    ReleaseOrderMismatch = 25,
}
