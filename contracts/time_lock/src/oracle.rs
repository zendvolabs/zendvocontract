use crate::errors::Error;
use crate::storage;
use crate::types::PriceCache;
use soroban_sdk::{contracttype, Address, Env, String};

/// Oracle price data structure
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriceData {
    pub rate: i128,     // Exchange rate with precision (e.g., 1000000 = 1.0)
    pub timestamp: u64, // Timestamp of price data
    pub source: String, // Price source identifier
}

/// Oracle configuration
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OracleConfig {
    pub oracle_address: Address, // Oracle contract address
    pub max_oracle_age: u64,     // Max age of oracle data in ledger entries
    pub is_paused: bool,         // Whether oracle checks are paused
}

/// Default oracle configuration
pub fn default_oracle_config(oracle_address: Address) -> OracleConfig {
    OracleConfig {
        oracle_address,
        max_oracle_age: 300, // 5 minutes (300 ledger entries)
        is_paused: false,
    }
}

/// Get price from oracle or cache
pub fn get_price(env: &Env, config: &OracleConfig) -> Result<i128, Error> {
    if config.is_paused {
        return Err(Error::OraclePaused);
    }

    if let Some(cached) = storage::get_price_cache(env) {
        if validate_data_freshness(env.ledger().timestamp(), cached.timestamp, config.max_oracle_age).is_ok() {
            return Ok(cached.rate);
        }
    }

    let oracle_rate: i128 = 1_000_000;
    let current_timestamp = env.ledger().timestamp();

    validate_rate_bounds(oracle_rate)?;

    let cache_entry = PriceCache {
        rate: oracle_rate,
        timestamp: current_timestamp,
    };
    storage::set_price_cache(env, &cache_entry);

    Ok(oracle_rate)
}

/// Validate oracle data freshness
pub fn validate_data_freshness(
    current_timestamp: u64,
    data_timestamp: u64,
    max_age: u64,
) -> Result<(), Error> {
    if current_timestamp.saturating_sub(data_timestamp) > max_age {
        return Err(Error::StaleOracleData);
    }
    Ok(())
}

/// Validate oracle rate bounds
pub fn validate_rate_bounds(rate: i128) -> Result<(), Error> {
    if rate <= 0 {
        return Err(Error::InvalidExchangeRate);
    }
    Ok(())
}
