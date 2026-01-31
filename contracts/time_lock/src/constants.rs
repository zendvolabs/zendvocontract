pub const GIFT_FEE_BPS: u32 = 200;
pub const MIN_GIFT_AMOUNT: i128 = 5_000_000;
pub const MAX_GIFT_AMOUNT: i128 = 1_000_000_000;
pub const MAX_PAYMENT_REF_LENGTH: u32 = 256;
/// Maximum lock duration: 10 years in seconds (sanity check for data entry errors)
pub const MAX_LOCK_DURATION: u64 = 10 * 365 * 24 * 60 * 60;
