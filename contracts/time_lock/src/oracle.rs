use soroban_sdk::{contracttype, Address, String};

/// Oracle price data structure
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriceData {
    pub rate: i128,              // Exchange rate with precision (e.g., 1000000 = 1.0)
    pub timestamp: u64,          // Timestamp of price data
    pub source: String,          // Price source identifier
}

/// Oracle configuration
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OracleConfig {
    pub oracle_address: Address,        // Oracle contract address
    pub max_oracle_age: u64,            // Max age of oracle data in ledger entries
    pub is_paused: bool,                // Whether oracle checks are paused
}

/// Default oracle configuration
pub fn default_oracle_config(oracle_address: Address) -> OracleConfig {
    OracleConfig {
        oracle_address,
        max_oracle_age: 300, // 5 minutes (300 ledger entries)
        is_paused: false,
    }
}

/// Validate oracle data freshness
pub fn validate_data_freshness(
    current_timestamp: u64,
    data_timestamp: u64,
    max_age: u64,
) -> Result<(), &'static str> {
    if current_timestamp.saturating_sub(data_timestamp) > max_age {
        return Err("Oracle data is stale");
    }
    Ok(())
}

/// Validate oracle rate bounds
pub fn validate_rate_bounds(rate: i128) -> Result<(), &'static str> {
    if rate <= 0 {
        return Err("Invalid exchange rate");
    }
    Ok(())
}
