use soroban_sdk::contracttype;
use soroban_sdk::Address;

/// Slippage configuration
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SlippageConfig {
    pub max_slippage_bps: u32,      // Maximum slippage in basis points (0-10000)
    pub admin: Address,              // Admin address for configuration
}

/// Default slippage configuration (2%)
pub fn default_slippage_config(admin: Address) -> SlippageConfig {
    SlippageConfig {
        max_slippage_bps: 200, // 2% default slippage
        admin,
    }
}

/// Validate slippage bounds
pub fn validate_slippage_bounds(slippage_bps: u32) -> Result<(), &'static str> {
    if slippage_bps > 10000 {
        return Err("Slippage exceeds maximum bounds (10000 bps)");
    }
    Ok(())
}

/// Calculate expected output with slippage
pub fn calculate_expected_output(
    oracle_rate: i128,
    amount: i128,
    max_slippage_bps: u32,
) -> i128 {
    let output = (amount as i128)
        .checked_mul(oracle_rate)
        .unwrap_or(0)
        / 1_000_000;

    let slippage_amount = output
        .checked_mul(max_slippage_bps as i128)
        .unwrap_or(0)
        / 10000;

    output - slippage_amount
}

/// Calculate percentage difference between two rates
pub fn calculate_rate_difference(oracle_rate: i128, actual_rate: i128) -> i128 {
    if oracle_rate == 0 {
        return 0;
    }
    let diff = (actual_rate as i128) - (oracle_rate as i128);
    (diff * 10000) / oracle_rate
}
