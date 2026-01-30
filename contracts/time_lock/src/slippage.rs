use crate::errors::Error;
use soroban_sdk::contracttype;
use soroban_sdk::{symbol_short, Address, Env};

/// Slippage configuration
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SlippageConfig {
    pub max_slippage_bps: u32, // Maximum slippage in basis points (0-10000)
    pub admin: Address,         // Admin address for configuration
}

/// Default slippage configuration (2%)
pub fn default_slippage_config(admin: Address) -> SlippageConfig {
    SlippageConfig {
        max_slippage_bps: 200, // 2% default slippage
        admin,
    }
}

/// Validate slippage bounds
pub fn validate_slippage_bounds(slippage_bps: u32) -> Result<(), Error> {
    if slippage_bps > 10000 {
        return Err(Error::InvalidSlippageConfig);
    }
    Ok(())
}


/// Calculate percentage difference between two rates
pub fn calculate_rate_difference(oracle_rate: i128, actual_rate: i128) -> i128 {
    if oracle_rate == 0 {
        return 0;
    }
    let diff = (actual_rate as i128) - (oracle_rate as i128);
    (diff * 10000) / oracle_rate
}

/// Validate slippage before transaction
pub fn validate_slippage(
    env: &Env,
    config: &SlippageConfig,
    oracle_rate: i128,
    actual_rate: i128,
) -> Result<(), Error> {
    let rate_diff = calculate_rate_difference(oracle_rate, actual_rate);

    if rate_diff.abs() > config.max_slippage_bps as i128 {
        env.events().publish(
            (symbol_short!("slip_f"),),
            (oracle_rate, actual_rate, config.max_slippage_bps),
        );
        return Err(Error::SlippageExceeded);
    }

    Ok(())
}
