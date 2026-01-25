#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Address, Map, Vec, symbol_short};

mod types;
mod errors;
mod constants;

#[cfg(test)]
mod simple_test;

use types::{Gift, GiftStatus, GiftUnlockedEvent};
use errors::Error;

#[contract]
pub struct TimeLockContract;

#[contractimpl]
impl TimeLockContract {
    // Storage keys for gifts map
    const GIFTS: soroban_sdk::Symbol = symbol_short!("GIFTS");
    const NEXT_GIFT_ID: soroban_sdk::Symbol = symbol_short!("NEXT_ID");

    /// Create a new time-locked gift
    pub fn create_gift(
        env: Env,
        sender: Address,
        recipient: Address,
        amount: i128,
        unlock_timestamp: u64,
    ) -> Result<u64, Error> {
        // Validate amount
        if amount < constants::MIN_GIFT_AMOUNT || amount > constants::MAX_GIFT_AMOUNT {
            return Err(Error::InvalidAmount);
        }

        // Get next gift ID
        let gift_id: u64 = env.storage().instance().get(&Self::NEXT_GIFT_ID).unwrap_or(0);
        let next_gift_id = gift_id + 1;
        env.storage().instance().set(&Self::NEXT_GIFT_ID, &next_gift_id);

        // Create gift
        let gift = Gift {
            sender: sender.clone(),
            recipient: recipient.clone(),
            amount,
            unlock_timestamp,
            status: GiftStatus::Created,
        };

        // Store gift
        let mut gifts: Map<u64, Gift> = env.storage().instance().get(&Self::GIFTS).unwrap_or(Map::new(&env));
        gifts.set(gift_id, gift);
        env.storage().instance().set(&Self::GIFTS, &gifts);

        Ok(gift_id)
    }

    /// Claim a gift (mark it as claimed but don't unlock yet)
    pub fn claim_gift(env: Env, gift_id: u64, recipient: Address) -> Result<(), Error> {
        let mut gifts: Map<u64, Gift> = env.storage().instance().get(&Self::GIFTS)
            .ok_or(Error::GiftNotFound)?;
        
        let mut gift = gifts.get(gift_id).ok_or(Error::GiftNotFound)?;

        // Verify recipient
        if gift.recipient != recipient {
            return Err(Error::Unauthorized);
        }

        // Check if already claimed
        if gift.status == GiftStatus::Claimed || gift.status == GiftStatus::Unlocked {
            return Err(Error::AlreadyClaimed);
        }

        // Update status to claimed
        gift.status = GiftStatus::Claimed;
        gifts.set(gift_id, gift);
        env.storage().instance().set(&Self::GIFTS, &gifts);

        Ok(())
    }

    /// Unlock a claimed gift if the unlock time has been reached
    pub fn unlock_gift(env: Env, gift_id: u64, recipient: Address) -> Result<(), Error> {
        let mut gifts: Map<u64, Gift> = env.storage().instance().get(&Self::GIFTS)
            .ok_or(Error::GiftNotFound)?;
        
        let mut gift = gifts.get(gift_id).ok_or(Error::GiftNotFound)?;

        // Verify recipient
        if gift.recipient != recipient {
            return Err(Error::Unauthorized);
        }

        // Check if gift is claimed
        if gift.status != GiftStatus::Claimed {
            return Err(Error::NotClaimed);
        }

        // Check if already unlocked
        if gift.status == GiftStatus::Unlocked {
            return Err(Error::AlreadyUnlocked);
        }

        // Get current ledger time
        let current_time = env.ledger().timestamp();

        // Check if unlock time has been reached
        if current_time < gift.unlock_timestamp {
            return Err(Error::UnlockTimeNotReached);
        }

        // Update status to unlocked
        let unlock_time = gift.unlock_timestamp;
        gift.status = GiftStatus::Unlocked;
        gifts.set(gift_id, gift);
        env.storage().instance().set(&Self::GIFTS, &gifts);

        // Emit GiftUnlocked event
        let event = GiftUnlockedEvent {
            gift_id,
            unlock_time,
            unlocked_at: current_time,
        };
        env.events().publish((gift_id, symbol_short!("UNLOCKED")), event);

        Ok(())
    }

    /// Get gift information
    pub fn get_gift(env: Env, gift_id: u64) -> Result<Gift, Error> {
        let gifts: Map<u64, Gift> = env.storage().instance().get(&Self::GIFTS)
            .ok_or(Error::GiftNotFound)?;
        
        gifts.get(gift_id).ok_or(Error::GiftNotFound)
    }

    /// Get time remaining until unlock (in seconds)
    pub fn get_time_remaining(env: Env, gift_id: u64) -> Result<u64, Error> {
        let gift = Self::get_gift(env.clone(), gift_id)?;
        
        if gift.status == GiftStatus::Unlocked {
            return Ok(0);
        }

        let current_time = env.ledger().timestamp();
        
        if current_time >= gift.unlock_timestamp {
            return Ok(0);
        }

        Ok(gift.unlock_timestamp - current_time)
    }

    /// Check if a gift can be unlocked
    pub fn can_unlock(env: Env, gift_id: u64) -> Result<bool, Error> {
        let gift = Self::get_gift(env.clone(), gift_id)?;
        
        if gift.status != GiftStatus::Claimed {
            return Ok(false);
        }

        let current_time = env.ledger().timestamp();
        Ok(current_time >= gift.unlock_timestamp)
    }

    /// Get all gifts for a recipient
    pub fn get_recipient_gifts(env: Env, recipient: Address) -> Result<Vec<u64>, Error> {
        let gifts: Map<u64, Gift> = env.storage().instance().get(&Self::GIFTS)
            .ok_or(Error::GiftNotFound)?;
        
        let mut recipient_gifts: Vec<u64> = Vec::new(&env);
        
        for (gift_id, gift) in gifts.iter() {
            if gift.recipient == recipient {
                recipient_gifts.push_back(gift_id);
            }
        }

        Ok(recipient_gifts)
    }
}
