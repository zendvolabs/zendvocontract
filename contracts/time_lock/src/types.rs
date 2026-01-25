use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GiftStatus {
    Created,
    Claimed,
    Unlocked,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Gift {
    pub sender: Address,
    pub recipient: Address,
    pub amount: i128,
    pub unlock_timestamp: u64,
    pub status: GiftStatus,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GiftUnlockedEvent {
    pub gift_id: u64,
    pub unlock_time: u64,
    pub unlocked_at: u64,
}
