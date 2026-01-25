use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotUnlocked = 1,
    AlreadyClaimed = 2,
    InvalidAmount = 3,
    Unauthorized = 4,
    GiftNotFound = 5,
    NotClaimed = 6,
    AlreadyUnlocked = 7,
    UnlockTimeNotReached = 8,
}
