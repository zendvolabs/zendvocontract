#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Bytes, Env, Address, BytesN, Symbol};

mod types;
mod errors;
mod constants;
mod test;

use types::{Gift, GiftStatus};
use errors::Error;
use constants::MIN_GIFT_AMOUNT;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GiftCreatedEvent {
    pub gift_id: BytesN<32>,
    pub sender: Address,
    pub amount: i128,
    pub unlock_time: u64,
    pub recipient_hash: BytesN<32>,
}

#[contract]
pub struct TimeLockContract;

#[contractimpl]
impl TimeLockContract {
    pub fn create_gift(
        env: Env,
        sender: Address,
        amount: i128,
        unlock_time: u64,
        recipient_phone_hash: BytesN<32>,
        usdc_token: Address,
    ) -> Result<BytesN<32>, Error> {
        sender.require_auth();

        if amount < MIN_GIFT_AMOUNT {
            return Err(Error::InvalidAmount);
        }

        let current_time = env.ledger().timestamp();
        if unlock_time <= current_time {
            return Err(Error::InvalidUnlockTime);
        }

        let mut hash_data = Bytes::new(&env);
        hash_data.append(&Bytes::from_slice(&env, &env.ledger().sequence().to_be_bytes()));
        hash_data.append(&Bytes::from_slice(&env, &current_time.to_be_bytes()));
        let gift_id: BytesN<32> = env.crypto().sha256(&hash_data).into();

        let token_client = token::Client::new(&env, &usdc_token);
        token_client.transfer_from(
            &env.current_contract_address(),
            &sender,
            &env.current_contract_address(),
            &amount,
        );

        let gift = Gift {
            sender: sender.clone(),
            amount,
            unlock_time,
            recipient_hash: recipient_phone_hash.clone(),
            status: GiftStatus::Created,
        };

        env.storage().persistent().set(&gift_id, &gift);

        env.events().publish(
            (Symbol::new(&env, "gift_created"),),
            GiftCreatedEvent {
                gift_id: gift_id.clone(),
                sender,
                amount,
                unlock_time,
                recipient_hash: recipient_phone_hash,
            },
        );

        Ok(gift_id)
    }

    pub fn get_gift(env: Env, gift_id: BytesN<32>) -> Result<Gift, Error> {
        env.storage()
            .persistent()
            .get(&gift_id)
            .ok_or(Error::GiftNotFound)
    }
}
