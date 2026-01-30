#[cfg(test)]
mod test {
    use crate::TimeLockContract;
    use soroban_sdk::{Env, testutils::{Ledger as TestLedger, Address as TestAddress}};
    use crate::types::{GiftStatus};
    use crate::constants;
    use crate::errors::Error;

    #[test]
    fn test_gift_creation() {
        let env = Env::default();
        let contract_id = env.register(TimeLockContract, ());
        
        let sender = <soroban_sdk::Address as TestAddress>::generate(&env);
        let recipient = <soroban_sdk::Address as TestAddress>::generate(&env);
        
        let current_time = 1000000u64;
        env.ledger().set_timestamp(current_time);
        
        let unlock_time = current_time + 3600;
        let amount = 10000000;

        // Test gift creation
        let gift_id = env.as_contract(&contract_id, || {
            TimeLockContract::create_gift(
                env.clone(),
                sender.clone(),
                recipient.clone(),
                amount,
                unlock_time,
            ).unwrap()
        });

        assert_eq!(gift_id, 0);

        // Test gift retrieval
        let gift = env.as_contract(&contract_id, || {
            TimeLockContract::get_gift(env.clone(), gift_id).unwrap()
        });
        assert_eq!(gift.sender, sender);
        assert_eq!(gift.recipient, recipient);
        assert_eq!(gift.amount, amount);
        assert_eq!(gift.unlock_timestamp, unlock_time);
        assert_eq!(gift.status, GiftStatus::Created);
    }

    #[test]
    fn test_gift_claim_flow() {
        let env = Env::default();
        let contract_id = env.register(TimeLockContract, ());
        
        let sender = <soroban_sdk::Address as TestAddress>::generate(&env);
        let recipient = <soroban_sdk::Address as TestAddress>::generate(&env);
        
        let current_time = 1000000u64;
        env.ledger().set_timestamp(current_time);
        
        let unlock_time = current_time + 3600;
        let amount = 10000000;

        // Create gift
        let gift_id = env.as_contract(&contract_id, || {
            TimeLockContract::create_gift(
                env.clone(),
                sender.clone(),
                recipient.clone(),
                amount,
                unlock_time,
            ).unwrap()
        });

        // Claim gift
        env.as_contract(&contract_id, || {
            TimeLockContract::claim_gift(env.clone(), gift_id, recipient.clone()).unwrap()
        });

        // Verify gift is claimed
        let gift = env.as_contract(&contract_id, || {
            TimeLockContract::get_gift(env.clone(), gift_id).unwrap()
        });
        assert_eq!(gift.status, GiftStatus::Claimed);

        // Try to claim again - should fail
        let result = env.as_contract(&contract_id, || {
            TimeLockContract::claim_gift(env.clone(), gift_id, recipient.clone())
        });
        assert_eq!(result, Err(Error::AlreadyClaimed));
    }

    #[test]
    fn test_unlock_time_boundary() {
        let env = Env::default();
        let contract_id = env.register(TimeLockContract, ());
        
        let sender = <soroban_sdk::Address as TestAddress>::generate(&env);
        let recipient = <soroban_sdk::Address as TestAddress>::generate(&env);
        
        let current_time = 1000000u64;
        env.ledger().set_timestamp(current_time);
        
        let unlock_time = current_time + 3600;
        let amount = 10000000;

        // Create and claim gift
        let gift_id = env.as_contract(&contract_id, || {
            let id = TimeLockContract::create_gift(
                env.clone(),
                sender.clone(),
                recipient.clone(),
                amount,
                unlock_time,
            ).unwrap();
            TimeLockContract::claim_gift(env.clone(), id, recipient.clone()).unwrap();
            id
        });

        // Try to unlock before time - should fail
        let result = env.as_contract(&contract_id, || {
            TimeLockContract::unlock_gift(env.clone(), gift_id, recipient.clone())
        });
        assert_eq!(result, Err(Error::UnlockTimeNotReached));

        // Advance to exactly unlock time
        env.ledger().set_timestamp(unlock_time);
        
        // Should succeed
        env.as_contract(&contract_id, || {
            TimeLockContract::unlock_gift(env.clone(), gift_id, recipient.clone()).unwrap()
        });
        
        let gift = env.as_contract(&contract_id, || {
            TimeLockContract::get_gift(env.clone(), gift_id).unwrap()
        });
        assert_eq!(gift.status, GiftStatus::Unlocked);
    }

    #[test]
    fn test_unlock_one_second_early_fails() {
        let env = Env::default();
        let contract_id = env.register(TimeLockContract, ());
        
        let sender = <soroban_sdk::Address as TestAddress>::generate(&env);
        let recipient = <soroban_sdk::Address as TestAddress>::generate(&env);
        
        let current_time = 1000000u64;
        env.ledger().set_timestamp(current_time);
        
        let unlock_time = current_time + 3600;
        let amount = 10000000;

        // Create and claim gift
        let gift_id = env.as_contract(&contract_id, || {
            let id = TimeLockContract::create_gift(
                env.clone(),
                sender.clone(),
                recipient.clone(),
                amount,
                unlock_time,
            ).unwrap();
            TimeLockContract::claim_gift(env.clone(), id, recipient.clone()).unwrap();
            id
        });

        // Try to unlock 1 second early
        env.ledger().set_timestamp(unlock_time - 1);
        
        let result = env.as_contract(&contract_id, || {
            TimeLockContract::unlock_gift(env.clone(), gift_id, recipient.clone())
        });
        assert_eq!(result, Err(Error::UnlockTimeNotReached));
    }

    #[test]
    fn test_time_remaining_calculations() {
        let env = Env::default();
        let contract_id = env.register(TimeLockContract, ());
        
        let sender = <soroban_sdk::Address as TestAddress>::generate(&env);
        let recipient = <soroban_sdk::Address as TestAddress>::generate(&env);
        
        let current_time = 1000000u64;
        env.ledger().set_timestamp(current_time);
        
        let unlock_time = current_time + 3600;
        let amount = 10000000;

        // Create and claim gift
        let gift_id = env.as_contract(&contract_id, || {
            let id = TimeLockContract::create_gift(
                env.clone(),
                sender.clone(),
                recipient.clone(),
                amount,
                unlock_time,
            ).unwrap();
            TimeLockContract::claim_gift(env.clone(), id, recipient.clone()).unwrap();
            id
        });

        // Check time remaining
        let time_remaining = env.as_contract(&contract_id, || {
            TimeLockContract::get_time_remaining(env.clone(), gift_id).unwrap()
        });
        assert_eq!(time_remaining, 3600);

        // Advance time by 1000 seconds
        env.ledger().set_timestamp(current_time + 1000);
        
        let time_remaining = env.as_contract(&contract_id, || {
            TimeLockContract::get_time_remaining(env.clone(), gift_id).unwrap()
        });
        assert_eq!(time_remaining, 2600);

        // Check can unlock
        let can_unlock = env.as_contract(&contract_id, || {
            TimeLockContract::can_unlock(env.clone(), gift_id).unwrap()
        });
        assert!(!can_unlock);

        // Advance past unlock time
        env.ledger().set_timestamp(unlock_time + 1);
        
        let time_remaining = env.as_contract(&contract_id, || {
            TimeLockContract::get_time_remaining(env.clone(), gift_id).unwrap()
        });
        assert_eq!(time_remaining, 0);

        let can_unlock = env.as_contract(&contract_id, || {
            TimeLockContract::can_unlock(env.clone(), gift_id).unwrap()
        });
        assert!(can_unlock);
    }

    #[test]
    fn test_amount_validation() {
        let env = Env::default();
        let contract_id = env.register(TimeLockContract, ());
        
        let sender = <soroban_sdk::Address as TestAddress>::generate(&env);
        let recipient = <soroban_sdk::Address as TestAddress>::generate(&env);
        
        let unlock_time = 1000000u64 + 3600;

        // Test amount too low
        let result = env.as_contract(&contract_id, || {
            TimeLockContract::create_gift(
                env.clone(),
                sender.clone(),
                recipient.clone(),
                constants::MIN_GIFT_AMOUNT - 1,
                unlock_time,
            )
        });
        assert_eq!(result, Err(Error::InvalidAmount));

        // Test amount too high
        let result = env.as_contract(&contract_id, || {
            TimeLockContract::create_gift(
                env.clone(),
                sender.clone(),
                recipient.clone(),
                constants::MAX_GIFT_AMOUNT + 1,
                unlock_time,
            )
        });
        assert_eq!(result, Err(Error::InvalidAmount));

        // Test valid amount
        let result = env.as_contract(&contract_id, || {
            TimeLockContract::create_gift(
                env.clone(),
                sender.clone(),
                recipient.clone(),
                constants::MIN_GIFT_AMOUNT,
                unlock_time,
            )
        });
        assert!(result.is_ok());
    }
}
