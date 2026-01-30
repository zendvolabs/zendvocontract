use soroban_sdk::{contracttype, Address, String};

/// Event emitted when oracle rate is queried
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OracleRateQueried {
    pub timestamp: u64,
    pub rate: i128,
    pub source: String,
}

/// Event emitted when slippage config is updated
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SlippageConfigUpdated {
    pub old_slippage: u32,
    pub new_slippage: u32,
    pub admin: Address,
}

/// Event emitted when slippage check fails
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SlippageCheckFailed {
    pub expected_rate: i128,
    pub actual_rate: i128,
    pub threshold: u32,
}

/// Event emitted when oracle address is updated
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OracleAddressUpdated {
    pub old_address: Address,
    pub new_address: Address,
}

/// Event topics
pub const EVENT_ORACLE_RATE_QUERIED: &[u8] = b"OracleRateQueried";
pub const EVENT_SLIPPAGE_CONFIG_UPDATED: &[u8] = b"SlippageConfigUpdated";
pub const EVENT_SLIPPAGE_CHECK_FAILED: &[u8] = b"SlippageCheckFailed";
pub const EVENT_ORACLE_ADDRESS_UPDATED: &[u8] = b"OracleAddressUpdated";
