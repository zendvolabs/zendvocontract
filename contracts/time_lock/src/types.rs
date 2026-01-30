use soroban_sdk::{contracttype, Address, String};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GiftStatus {
    Created = 0,
    Claimed = 1,
    Withdrawn = 2,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GiftStatus {
    Created,
    Claimed,
    Unlocked,
    Withdrawn,
    Refunded,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Gift {
    pub sender: Address,
    pub recipient: Option<Address>,
    pub amount: i128,
    pub unlock_timestamp: u64,
    pub recipient_phone_hash: String,
    pub status: GiftStatus,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriceCache {
    pub rate: i128,
    pub timestamp: u64,
}
