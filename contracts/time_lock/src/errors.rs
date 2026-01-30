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
    OracleUnavailable = 8,
    StaleOracleData = 9,
    InvalidExchangeRate = 10,
    SlippageExceeded = 11,
    InvalidSlippageConfig = 12,
    OraclePaused = 13,
    InsufficientLiquidity = 14,
    InsufficientFunds = 15,
    InsufficientAllowance = 16,
    InvalidTokenAddress = 17,
    TransferFailed = 18,
}
