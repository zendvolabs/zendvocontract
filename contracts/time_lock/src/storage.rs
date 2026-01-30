use crate::oracle::OracleConfig;
use crate::slippage::SlippageConfig;
use crate::types::{Gift, PriceCache};
use soroban_sdk::{contracttype, Address, BytesN, Env, String};

const DAY_IN_LEDGERS: u32 = 17280;
const INSTANCE_BUMP_AMOUNT: u32 = 7 * DAY_IN_LEDGERS;
const INSTANCE_LIFETIME_THRESHOLD: u32 = DAY_IN_LEDGERS;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    OracleAuthKey,
    OracleConfig,
    SlippageConfig,
    NextGiftId,
    Gift(u64),
    PriceCache,
    UsdcAddress,
    TotalHeld,
    TotalGifted,
    TotalFees,
    PaymentReference(String),
}

pub fn extend_instance_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Admin)
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
    extend_instance_ttl(env);
}

pub fn has_admin(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Admin)
}

pub fn get_oracle_auth_key(env: &Env) -> BytesN<32> {
    env.storage()
        .instance()
        .get(&DataKey::OracleAuthKey)
        .expect("Contract not initialized")
}

pub fn set_oracle_auth_key(env: &Env, key: &BytesN<32>) {
    env.storage().instance().set(&DataKey::OracleAuthKey, key);
    extend_instance_ttl(env);
}

pub fn get_oracle_config(env: &Env) -> Option<OracleConfig> {
    env.storage().instance().get(&DataKey::OracleConfig)
}

pub fn set_oracle_config(env: &Env, config: &OracleConfig) {
    env.storage().instance().set(&DataKey::OracleConfig, config);
    extend_instance_ttl(env);
}

pub fn get_slippage_config(env: &Env) -> Option<SlippageConfig> {
    env.storage().instance().get(&DataKey::SlippageConfig)
}

pub fn set_slippage_config(env: &Env, config: &SlippageConfig) {
    env.storage()
        .instance()
        .set(&DataKey::SlippageConfig, config);
    extend_instance_ttl(env);
}

pub fn get_next_gift_id(env: &Env) -> u64 {
    env.storage().instance().get(&DataKey::NextGiftId).unwrap_or(1)
}

pub fn increment_next_gift_id(env: &Env) -> u64 {
    let id = get_next_gift_id(env);
    env.storage().instance().set(&DataKey::NextGiftId, &(id + 1));
    extend_instance_ttl(env);
    id
}

pub fn get_gift(env: &Env, id: u64) -> Option<Gift> {
    env.storage().instance().get(&DataKey::Gift(id))
}

pub fn set_gift(env: &Env, id: u64, gift: &Gift) {
    env.storage().instance().set(&DataKey::Gift(id), gift);
    extend_instance_ttl(env);
}


pub fn get_price_cache(env: &Env) -> Option<PriceCache> {
    env.storage().instance().get(&DataKey::PriceCache)
}

pub fn set_price_cache(env: &Env, cache: &PriceCache) {
    env.storage().instance().set(&DataKey::PriceCache, cache);
    extend_instance_ttl(env);
}

// USDC Token address
pub fn get_usdc_address(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::UsdcAddress)
}

pub fn set_usdc_address(env: &Env, address: &Address) {
    env.storage().instance().set(&DataKey::UsdcAddress, address);
    extend_instance_ttl(env);
}

// Internal Balance Tracking
pub fn get_total_held(env: &Env) -> i128 {
    env.storage().instance().get(&DataKey::TotalHeld).unwrap_or(0)
}

pub fn set_total_held(env: &Env, amount: i128) {
    env.storage().instance().set(&DataKey::TotalHeld, &amount);
    extend_instance_ttl(env);
}

pub fn get_total_gifted(env: &Env) -> i128 {
    env.storage().instance().get(&DataKey::TotalGifted).unwrap_or(0)
}

pub fn set_total_gifted(env: &Env, amount: i128) {
    env.storage().instance().set(&DataKey::TotalGifted, &amount);
    extend_instance_ttl(env);
}

pub fn get_total_fees(env: &Env) -> i128 {
    env.storage().instance().get(&DataKey::TotalFees).unwrap_or(0)
}

pub fn set_total_fees(env: &Env, amount: i128) {
    env.storage().instance().set(&DataKey::TotalFees, &amount);
    extend_instance_ttl(env);
}

// Payment Reference tracking
pub fn get_payment_reference_gift_id(env: &Env, payment_ref: &String) -> Option<u64> {
    env.storage()
        .instance()
        .get(&DataKey::PaymentReference(payment_ref.clone()))
}

pub fn set_payment_reference_gift_id(env: &Env, payment_ref: &String, gift_id: u64) {
    env.storage()
        .instance()
        .set(&DataKey::PaymentReference(payment_ref.clone()), &gift_id);
    extend_instance_ttl(env);
}

pub fn has_payment_reference(env: &Env, payment_ref: &String) -> bool {
    env.storage()
        .instance()
        .has(&DataKey::PaymentReference(payment_ref.clone()))
}
