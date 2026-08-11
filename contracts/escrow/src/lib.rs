#![no_std]

mod contract;
mod core {
    pub mod dispute;
    pub mod escrow;
    pub mod milestone;
    pub use dispute::*;
    pub use escrow::*;
    pub use milestone::*;
    pub mod validators {
        pub mod dispute;
        pub mod escrow;
        pub mod milestone;
    }
}
mod error;
mod events {
    pub mod handler;
}
mod modules {
    pub mod math {
        pub mod basic;
        pub mod safe;

        pub use basic::*;
        pub use safe::*;
    }

    pub mod fee {
        pub mod calculator;
        pub mod distribution;

        pub use calculator::*;
    }
}

/// This module is currently Work In Progress.
mod storage {
    pub mod types;
}

#[cfg(test)]
mod tests;

pub use crate::contract::EscrowContract;
