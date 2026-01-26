use soroban_sdk::{contracttype, Address, BytesN};

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
    pub amount: i128,
    pub unlock_time: u64,
    pub recipient_hash: BytesN<32>,
    pub status: GiftStatus,
}
