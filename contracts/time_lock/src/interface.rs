use crate::errors::Error;
use crate::oracle::OracleConfig;
use crate::slippage::SlippageConfig;
use crate::types::Gift;
use soroban_sdk::{Address, BytesN, Env, String};

pub trait TimeLockTrait {
    /// Initialize contract with admin, oracle auth key (for claims), and oracle config (for price feed)
    fn initialize(
        env: Env,
        admin: Address,
        oracle_pk: BytesN<32>,
        oracle_address: Address,
        usdc_address: Address,
    ) -> Result<(), Error>;

    fn create_gift(
        env: Env,
        sender: Address,
        amount: i128,
        unlock_timestamp: u64,
        recipient_phone_hash: String,
    ) -> Result<u64, Error>;

    fn claim_gift(
        env: Env,
        claimant: Address,
        gift_id: u64,
        verification_proof: BytesN<64>,
    ) -> Result<(), Error>;

    fn withdraw_to_bank(
        env: Env,
        gift_id: u64,
        recipient_bank_details_hash: String,
        anchor_address: Address,
    ) -> Result<(), Error>;

    fn set_oracle_address(env: Env, new_oracle_address: Address) -> Result<(), Error>;

    fn set_max_oracle_age(env: Env, max_age: u64) -> Result<(), Error>;

    fn set_oracle_paused(env: Env, paused: bool) -> Result<(), Error>;

    fn set_max_slippage(env: Env, slippage_bps: u32) -> Result<(), Error>;

    fn check_exchange_rate(env: Env, currency_pair: String) -> Result<i128, Error>;

    fn validate_slippage(env: Env, oracle_rate: i128, actual_rate: i128) -> Result<(), Error>;

    fn get_oracle_config(env: Env) -> Result<OracleConfig, Error>;

    fn get_slippage_config(env: Env) -> Result<SlippageConfig, Error>;

    fn get_gift(env: Env, gift_id: u64) -> Result<Gift, Error>;

    /// SEP-41 Wrapper: Get USDC balance of an address
    fn get_balance(env: Env, owner: Address) -> Result<i128, Error>;

    /// Internal Tracking: Get total USDC held by contract
    fn get_total_held(env: Env) -> Result<i128, Error>;

    /// Internal Tracking: Get accumulated platform fees
    fn get_total_fees(env: Env) -> Result<i128, Error>;

    /// Create gift from USDC deposit (oracle-only)
    /// Called by backend after Stripe payment confirmation and Anchor USDC deposit
    fn deposit_and_create_gift(
        env: Env,
        payment_reference: String,
        amount: i128,
        unlock_timestamp: u64,
        recipient_phone_hash: String,
    ) -> Result<u64, Error>;

    /// Get gift ID by payment reference
    fn get_gift_by_payment_reference(
        env: Env,
        payment_reference: String,
    ) -> Result<u64, Error>;
}
