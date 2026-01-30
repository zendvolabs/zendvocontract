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
    InvalidStatus = 6,
    InvalidProof = 7,
    OracleUnavailable = 5,
    StaleOracleData = 6,
    InvalidExchangeRate = 7,
    SlippageExceeded = 8,
    InvalidSlippageConfig = 9,
    OraclePaused = 10,
    InsufficientLiquidity = 11,
}
