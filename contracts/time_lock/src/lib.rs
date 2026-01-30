#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, xdr::ToXdr, Address, Bytes, BytesN, Env,
    String,
};

mod types;
mod errors;
mod constants;
mod oracle;
mod slippage;
mod events;
mod test;

use types::{Gift, GiftStatus};
use errors::Error;

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Gift(u64),
    NextGiftId,
    Oracle, // Stores BytesN<32> (Ed25519 Public Key)
use errors::Error;
use oracle::OracleConfig;
use slippage::SlippageConfig;
use events::*;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriceCache {
    pub rate: i128,
    pub timestamp: u64,
}

#[contract]
pub struct TimeLockContract;

/// Helper: Get admin from storage
fn get_admin(env: &Env) -> Result<Address, Error> {
    env.storage()
        .persistent()
        .get::<_, Address>(&symbol_short!("admin"))
        .ok_or(Error::Unauthorized)
}

/// Helper: Get oracle config from storage
fn get_oracle_config(env: &Env) -> Result<OracleConfig, Error> {
    env.storage()
        .persistent()
        .get::<_, OracleConfig>(&symbol_short!("oracle"))
        .ok_or(Error::OracleUnavailable)
}

/// Helper: Get slippage config from storage
fn get_slippage_config_internal(env: &Env) -> Result<SlippageConfig, Error> {
    env.storage()
        .persistent()
        .get::<_, SlippageConfig>(&symbol_short!("slippage"))
        .ok_or(Error::Unauthorized)
}

/// Helper: Verify admin auth and return admin address
fn require_admin_auth(env: &Env) -> Result<Address, Error> {
    let admin = get_admin(env)?;
    admin.require_auth();
    Ok(admin)
}

#[contractimpl]
impl TimeLockContract {
    pub fn initialize(env: Env, oracle_pk: BytesN<32>) {
        if env.storage().instance().has(&DataKey::Oracle) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Oracle, &oracle_pk);
        env.storage().instance().set(&DataKey::NextGiftId, &1u64);
    }

    pub fn create_gift(
        env: Env,
        sender: Address,
        amount: i128,
        unlock_timestamp: u64,
        recipient_phone_hash: String,
    ) -> u64 {
        sender.require_auth();

        // Check amount limits
        if amount < constants::MIN_GIFT_AMOUNT || amount > constants::MAX_GIFT_AMOUNT {
            panic!("Invalid amount");
        }

        let gift_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextGiftId)
            .unwrap_or(1);

        let gift = Gift {
            sender,
            recipient: None,
            amount,
            unlock_timestamp,
            recipient_phone_hash,
            status: GiftStatus::Created,
        };

        env.storage().instance().set(&DataKey::Gift(gift_id), &gift);
        env.storage()
            .instance()
            .set(&DataKey::NextGiftId, &(gift_id + 1));

        gift_id
    }

    pub fn claim_gift(
        env: Env,
        claimant: Address,
        gift_id: u64,
        verification_proof: BytesN<64>,
    ) -> Result<(), Error> {
        claimant.require_auth();

        let key = DataKey::Gift(gift_id);
        if !env.storage().instance().has(&key) {
            return Err(Error::GiftNotFound);
        }

        let mut gift: Gift = env.storage().instance().get(&key).unwrap();

        // Verify status
        if gift.status != GiftStatus::Created {
            if gift.status == GiftStatus::Claimed {
                return Err(Error::AlreadyClaimed);
            }
            return Err(Error::InvalidStatus);
        }
        
        // Verify Unlock Time
        if env.ledger().timestamp() < gift.unlock_timestamp {
             return Err(Error::NotUnlocked);
        }

        // Verify Oracle Proof
        let oracle_pk: BytesN<32> = env
            .storage()
            .instance()
            .get(&DataKey::Oracle)
            .expect("Contract not initialized");

        // Construct payload: claimant XDR + recipient_phone_hash XDR
        let mut payload = Bytes::new(&env);
        payload.append(&claimant.clone().to_xdr(&env));
        payload.append(&gift.recipient_phone_hash.clone().to_xdr(&env));

        // Verify signature
        env.crypto()
            .ed25519_verify(&oracle_pk, &payload, &verification_proof);

        // Update Gift
        gift.recipient = Some(claimant.clone());
        gift.status = GiftStatus::Claimed;

        env.storage().instance().set(&key, &gift);

        // Emit event
        env.events().publish(
            (symbol_short!("claimed"),),
            (gift_id, claimant, env.ledger().timestamp()),
        );

    /// Initialize contract with admin and oracle address
    pub fn initialize(env: Env, admin: Address, oracle_address: Address) -> Result<(), Error> {
        if env.storage().persistent().has(&symbol_short!("admin")) {
            return Err(Error::Unauthorized);
        }

        env.storage()
            .persistent()
            .set(&symbol_short!("admin"), &admin);

        let oracle_config = oracle::default_oracle_config(oracle_address);
        env.storage()
            .persistent()
            .set(&symbol_short!("oracle"), &oracle_config);

        let slippage_config = slippage::default_slippage_config(admin.clone());
        env.storage()
            .persistent()
            .set(&symbol_short!("slippage"), &slippage_config);

        Ok(())
    }

    /// Get current oracle configuration (public view)
    pub fn get_oracle_status(env: Env) -> Result<OracleConfig, Error> {
        env.storage()
            .persistent()
            .get::<_, OracleConfig>(&symbol_short!("oracle"))
            .ok_or(Error::OracleUnavailable)
    }

    /// Set oracle address (admin only)
    pub fn set_oracle_address(env: Env, new_oracle_address: Address) -> Result<(), Error> {
        let _admin = require_admin_auth(&env)?;

        let mut oracle_config = get_oracle_config(&env)?;
        let old_address = oracle_config.oracle_address.clone();
        oracle_config.oracle_address = new_oracle_address.clone();

        env.storage()
            .persistent()
            .set(&symbol_short!("oracle"), &oracle_config);

        env.events().publish(
            (symbol_short!("oracle_ad"),),
            OracleAddressUpdated {
                old_address,
                new_address: new_oracle_address,
            },
        );

        Ok(())
    }

    /// Set maximum oracle data age (admin only)
    pub fn set_max_oracle_age(env: Env, max_age: u64) -> Result<(), Error> {
        let _admin = require_admin_auth(&env)?;

        let mut oracle_config = get_oracle_config(&env)?;
        oracle_config.max_oracle_age = max_age;

        env.storage()
            .persistent()
            .set(&symbol_short!("oracle"), &oracle_config);

        Ok(())
    }

    /// Pause oracle checks (emergency admin function)
    pub fn pause_oracle_checks(env: Env) -> Result<(), Error> {
        let _admin = require_admin_auth(&env)?;

        let mut oracle_config = get_oracle_config(&env)?;
        oracle_config.is_paused = true;

        env.storage()
            .persistent()
            .set(&symbol_short!("oracle"), &oracle_config);

        Ok(())
    }

    /// Resume oracle checks (admin function)
    pub fn resume_oracle_checks(env: Env) -> Result<(), Error> {
        let _admin = require_admin_auth(&env)?;

        let mut oracle_config = get_oracle_config(&env)?;
        oracle_config.is_paused = false;

        env.storage()
            .persistent()
            .set(&symbol_short!("oracle"), &oracle_config);

        Ok(())
    }

    /// Set maximum slippage (admin only)
    pub fn set_max_slippage(env: Env, slippage_bps: u32) -> Result<(), Error> {
        slippage::validate_slippage_bounds(slippage_bps)
            .map_err(|_| Error::InvalidSlippageConfig)?;

        let admin = require_admin_auth(&env)?;

        let mut slippage_config = get_slippage_config_internal(&env)?;
        let old_slippage = slippage_config.max_slippage_bps;
        slippage_config.max_slippage_bps = slippage_bps;

        env.storage()
            .persistent()
            .set(&symbol_short!("slippage"), &slippage_config);

        env.events().publish(
            (symbol_short!("slip_upd"),),
            SlippageConfigUpdated {
                old_slippage,
                new_slippage: slippage_bps,
                admin,
            },
        );

        Ok(())
    }

    /// Get current slippage configuration
    pub fn get_slippage_config(env: Env) -> Result<SlippageConfig, Error> {
        get_slippage_config_internal(&env)
    }

    /// Query current exchange rate from cache or oracle
    /// Returns rate with precision factor (1000000 = 1.0)
    pub fn check_exchange_rate(env: Env, _currency_pair: String) -> Result<i128, Error> {
        let oracle_config = get_oracle_config(&env)?;

        if oracle_config.is_paused {
            return Err(Error::OraclePaused);
        }

        // Try to get cached price first
        if let Some(cached) = env
            .storage()
            .temporary()
            .get::<_, PriceCache>(&symbol_short!("price"))
        {
            let current_ledger = env.ledger().timestamp();
            if current_ledger.saturating_sub(cached.timestamp) < oracle_config.max_oracle_age {
                return Ok(cached.rate);
            }
        }

        // Placeholder for actual oracle call
        let oracle_rate: i128 = 1_000_000; // 1.0 USDC/NGN (placeholder)
        let current_timestamp = env.ledger().timestamp();

        // Validate rate bounds
        oracle::validate_rate_bounds(oracle_rate).map_err(|_| Error::InvalidExchangeRate)?;

        // Cache the rate
        env.storage().temporary().set(
            &symbol_short!("price"),
            &PriceCache {
                rate: oracle_rate,
                timestamp: current_timestamp,
            },
        );

        env.events().publish(
            (symbol_short!("price_q"),),
            OracleRateQueried {
                timestamp: current_timestamp,
                rate: oracle_rate,
                source: String::from_str(&env, "oracle"),
            },
        );

        Ok(oracle_rate)
    }

    /// Validate slippage before transaction
    /// Returns error if slippage exceeds threshold
    pub fn validate_slippage(env: Env, oracle_rate: i128, actual_rate: i128) -> Result<(), Error> {
        let slippage_config = get_slippage_config_internal(&env)?;
        let rate_diff = slippage::calculate_rate_difference(oracle_rate, actual_rate);

        if rate_diff.abs() > slippage_config.max_slippage_bps as i128 {
            env.events().publish(
                (symbol_short!("slip_f"),),
                SlippageCheckFailed {
                    expected_rate: oracle_rate,
                    actual_rate,
                    threshold: slippage_config.max_slippage_bps,
                },
            );
            return Err(Error::SlippageExceeded);
        }

        Ok(())
    }
}

