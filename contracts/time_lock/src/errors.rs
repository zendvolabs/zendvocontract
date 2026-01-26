use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotUnlocked = 1,
    AlreadyClaimed = 2,
    InvalidAmount = 3,
    Unauthorized = 4,
    InvalidUnlockTime = 5,
    GiftNotFound = 6,
    TransferFailed = 7,
}
