#[cfg(test)]
mod test {
    use crate::events::{WithdrawalSuccess, FeeWithdrawal};
    use crate::TimeLockContract;
    use soroban_sdk::{Env, String, testutils::{Ledger as TestLedger, Address as TestAddress, Events}, BytesN};
    use crate::types::{GiftStatus};
    use crate::constants;
    use crate::errors::Error;

    fn setup_test_env() -> (Env, soroban_sdk::Address, soroban_sdk::Address, soroban_sdk::Address, soroban_sdk::Address, BytesN<32>) {
        let env = Env::default();
        env.mock_all_auths();

        let admin = <soroban_sdk::Address as TestAddress>::generate(&env);
        let oracle_address = <soroban_sdk::Address as TestAddress>::generate(&env);
        let usdc_address = <soroban_sdk::Address as TestAddress>::generate(&env);

        // Generate a dummy oracle public key
        let oracle_pk = BytesN::from_array(&env, &[0u8; 32]);

        (env, admin, oracle_address, usdc_address, oracle_pk, oracle_pk)
    }

    #[test]
    fn test_gift_creation() {
        let (env, admin, oracle_address, usdc_address, oracle_pk, _) = setup_test_env();
        let contract_id = env.register(TimeLockContract, ());

        // Initialize contract
        env.as_contract(&contract_id, || {
            TimeLockContract::initialize(
                env.clone(),
                admin.clone(),
                oracle_pk.clone(),
                oracle_address,
                usdc_address,
            ).unwrap();
        });

        let sender = <soroban_sdk::Address as TestAddress>::generate(&env);
        let current_time = 1000000u64;
        env.ledger().set_timestamp(current_time);

        let unlock_time = current_time + 3600;
        let amount = 10000000i128;
        let phone_hash = String::from_str(&env, "+1234567890");

        // Test gift creation
        let gift_id = env.as_contract(&contract_id, || {
            TimeLockContract::create_gift(
                env.clone(),
                sender.clone(),
                amount,
                unlock_time,
                phone_hash.clone(),
            ).unwrap()
        });

        assert_eq!(gift_id, 1);

        // Test gift retrieval
        let gift = env.as_contract(&contract_id, || {
            TimeLockContract::get_gift(env.clone(), gift_id).unwrap()
        });
        assert_eq!(gift.sender, sender);
        assert_eq!(gift.recipient, None);
        assert_eq!(gift.amount, amount);
        assert_eq!(gift.unlock_timestamp, unlock_time);
        assert_eq!(gift.status, GiftStatus::Created);
    }

    #[test]
    fn test_gift_claim_flow() {
        let (env, admin, oracle_address, usdc_address, oracle_pk, _) = setup_test_env();
        let contract_id = env.register(TimeLockContract, ());

        env.as_contract(&contract_id, || {
            TimeLockContract::initialize(
                env.clone(),
                admin.clone(),
                oracle_pk.clone(),
                oracle_address,
                usdc_address,
            ).unwrap();
        });

        let sender = <soroban_sdk::Address as TestAddress>::generate(&env);
        let claimant = <soroban_sdk::Address as TestAddress>::generate(&env);

        let current_time = 1000000u64;
        env.ledger().set_timestamp(current_time);

        let unlock_time = current_time + 3600;
        let amount = 10000000i128;
        let phone_hash = String::from_str(&env, "+1234567890");

        // Create gift
        let gift_id = env.as_contract(&contract_id, || {
            TimeLockContract::create_gift(
                env.clone(),
                sender.clone(),
                amount,
                unlock_time,
                phone_hash.clone(),
            ).unwrap()
        });

        // Advance time to unlock
        env.ledger().set_timestamp(unlock_time);

        // Claim gift with verification proof
        let verification_proof = BytesN::from_array(&env, &[0u8; 64]);
        env.as_contract(&contract_id, || {
            TimeLockContract::claim_gift(env.clone(), claimant.clone(), gift_id, verification_proof).unwrap()
        });

        // Verify gift is unlocked
        let gift = env.as_contract(&contract_id, || {
            TimeLockContract::get_gift(env.clone(), gift_id).unwrap()
        });
        assert_eq!(gift.status, GiftStatus::Unlocked);
        assert_eq!(gift.recipient, Some(claimant.clone()));

        // Try to claim again - should fail
        let result = env.as_contract(&contract_id, || {
            TimeLockContract::claim_gift(env.clone(), claimant.clone(), gift_id, verification_proof)
        });
        assert_eq!(result, Err(Error::InvalidStatus));
    }

    #[test]
    fn test_claim_before_unlock_fails() {
        let (env, admin, oracle_address, usdc_address, oracle_pk, _) = setup_test_env();
        let contract_id = env.register(TimeLockContract, ());

        env.as_contract(&contract_id, || {
            TimeLockContract::initialize(
                env.clone(),
                admin,
                oracle_pk.clone(),
                oracle_address,
                usdc_address,
            ).unwrap();
        });

        let sender = <soroban_sdk::Address as TestAddress>::generate(&env);
        let claimant = <soroban_sdk::Address as TestAddress>::generate(&env);

        let current_time = 1000000u64;
        env.ledger().set_timestamp(current_time);

        let unlock_time = current_time + 3600;
        let amount = 10000000i128;
        let phone_hash = String::from_str(&env, "+1234567890");

        let gift_id = env.as_contract(&contract_id, || {
            TimeLockContract::create_gift(
                env.clone(),
                sender.clone(),
                amount,
                unlock_time,
                phone_hash.clone(),
            ).unwrap()
        });

        // Try to claim before unlock time
        let verification_proof = BytesN::from_array(&env, &[0u8; 64]);
        let result = env.as_contract(&contract_id, || {
            TimeLockContract::claim_gift(env.clone(), claimant.clone(), gift_id, verification_proof)
        });
        assert_eq!(result, Err(Error::NotUnlocked));
    }

    #[test]
    fn test_amount_validation() {
        let (env, admin, oracle_address, usdc_address, oracle_pk, _) = setup_test_env();
        let contract_id = env.register(TimeLockContract, ());

        env.as_contract(&contract_id, || {
            TimeLockContract::initialize(
                env.clone(),
                admin,
                oracle_pk,
                oracle_address,
                usdc_address,
            ).unwrap();
        });

        let sender = <soroban_sdk::Address as TestAddress>::generate(&env);
        let unlock_time = 1000000u64 + 3600;
        let phone_hash = String::from_str(&env, "+1234567890");

        // Test amount too low
        let result = env.as_contract(&contract_id, || {
            TimeLockContract::create_gift(
                env.clone(),
                sender.clone(),
                constants::MIN_GIFT_AMOUNT - 1,
                unlock_time,
                phone_hash.clone(),
            )
        });
        assert_eq!(result, Err(Error::InvalidAmount));

        // Test amount too high
        let result = env.as_contract(&contract_id, || {
            TimeLockContract::create_gift(
                env.clone(),
                sender.clone(),
                constants::MAX_GIFT_AMOUNT + 1,
                unlock_time,
                phone_hash.clone(),
            )
        });
        assert_eq!(result, Err(Error::InvalidAmount));

        // Test valid amount
        let result = env.as_contract(&contract_id, || {
            TimeLockContract::create_gift(
                env.clone(),
                sender.clone(),
                constants::MIN_GIFT_AMOUNT,
                unlock_time,
                phone_hash.clone(),
            )
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_withdraw_gift_flow() {
        let (env, admin, oracle_address, usdc_address, oracle_pk, _) = setup_test_env();
        let contract_id = env.register(TimeLockContract, ());

        env.as_contract(&contract_id, || {
            TimeLockContract::initialize(
                env.clone(),
                admin,
                oracle_pk.clone(),
                oracle_address,
                usdc_address,
            ).unwrap();
        });

        let sender = <soroban_sdk::Address as TestAddress>::generate(&env);
        let claimant = <soroban_sdk::Address as TestAddress>::generate(&env);

        let current_time = 1000000u64;
        env.ledger().set_timestamp(current_time);

        let unlock_time = current_time + 3600;
        let amount = 10000000i128;
        let phone_hash = String::from_str(&env, "+1234567890");

        // Create gift
        let gift_id = env.as_contract(&contract_id, || {
            TimeLockContract::create_gift(
                env.clone(),
                sender.clone(),
                amount,
                unlock_time,
                phone_hash.clone(),
            ).unwrap()
        });

        // Advance time to unlock
        env.ledger().set_timestamp(unlock_time);

        // Claim gift
        let verification_proof = BytesN::from_array(&env, &[0u8; 64]);
        env.as_contract(&contract_id, || {
            TimeLockContract::claim_gift(env.clone(), claimant.clone(), gift_id, verification_proof).unwrap()
        });

        // Withdraw gift
        env.as_contract(&contract_id, || {
            TimeLockContract::withdraw_gift(env.clone(), gift_id).unwrap()
        });

        // Verify gift status is Withdrawn
        let gift = env.as_contract(&contract_id, || {
            TimeLockContract::get_gift(env.clone(), gift_id).unwrap()
        });
        assert_eq!(gift.status, GiftStatus::Withdrawn);

        // Verify fee calculation (2% = 200bps)
        let expected_fee = (amount * constants::GIFT_FEE_BPS as i128) / 10000;
        let expected_withdrawal = amount - expected_fee;
        assert_eq!(expected_fee, 200000); // 2% of 10000000
        assert_eq!(expected_withdrawal, 9800000); // 98% of 10000000

        // Verify total fees accumulated
        let total_fees = env.as_contract(&contract_id, || {
            TimeLockContract::get_total_fees(env.clone()).unwrap()
        });
        assert_eq!(total_fees, expected_fee);

        // Verify events were emitted
        let events = env.events().all();
        let withdrawal_event = events.iter().find(|e| {
            e.topics.len() > 0 && e.topics.get(0).unwrap() == &soroban_sdk::symbol_short!("withdr_s")
        });
        assert!(withdrawal_event.is_some());
    }

    #[test]
    fn test_withdraw_before_unlock_fails() {
        let (env, admin, oracle_address, usdc_address, oracle_pk, _) = setup_test_env();
        let contract_id = env.register(TimeLockContract, ());

        env.as_contract(&contract_id, || {
            TimeLockContract::initialize(
                env.clone(),
                admin,
                oracle_pk.clone(),
                oracle_address,
                usdc_address,
            ).unwrap();
        });

        let sender = <soroban_sdk::Address as TestAddress>::generate(&env);

        let current_time = 1000000u64;
        env.ledger().set_timestamp(current_time);

        let unlock_time = current_time + 3600;
        let amount = 10000000i128;
        let phone_hash = String::from_str(&env, "+1234567890");

        let gift_id = env.as_contract(&contract_id, || {
            TimeLockContract::create_gift(
                env.clone(),
                sender.clone(),
                amount,
                unlock_time,
                phone_hash.clone(),
            ).unwrap()
        });

        // Try to withdraw without claiming - should fail
        let result = env.as_contract(&contract_id, || {
            TimeLockContract::withdraw_gift(env.clone(), gift_id)
        });
        assert_eq!(result, Err(Error::InvalidStatus));
    }

    #[test]
    fn test_withdraw_twice_fails() {
        let (env, admin, oracle_address, usdc_address, oracle_pk, _) = setup_test_env();
        let contract_id = env.register(TimeLockContract, ());

        env.as_contract(&contract_id, || {
            TimeLockContract::initialize(
                env.clone(),
                admin,
                oracle_pk.clone(),
                oracle_address,
                usdc_address,
            ).unwrap();
        });

        let sender = <soroban_sdk::Address as TestAddress>::generate(&env);
        let claimant = <soroban_sdk::Address as TestAddress>::generate(&env);

        let current_time = 1000000u64;
        env.ledger().set_timestamp(current_time);

        let unlock_time = current_time + 3600;
        let amount = 10000000i128;
        let phone_hash = String::from_str(&env, "+1234567890");

        let gift_id = env.as_contract(&contract_id, || {
            TimeLockContract::create_gift(
                env.clone(),
                sender.clone(),
                amount,
                unlock_time,
                phone_hash.clone(),
            ).unwrap()
        });

        env.ledger().set_timestamp(unlock_time);

        let verification_proof = BytesN::from_array(&env, &[0u8; 64]);
        env.as_contract(&contract_id, || {
            TimeLockContract::claim_gift(env.clone(), claimant.clone(), gift_id, verification_proof).unwrap();
            TimeLockContract::withdraw_gift(env.clone(), gift_id).unwrap();
        });

        // Try to withdraw again
        let result = env.as_contract(&contract_id, || {
            TimeLockContract::withdraw_gift(env.clone(), gift_id)
        });
        assert_eq!(result, Err(Error::InvalidStatus));
    }

    #[test]
    fn test_withdraw_accumulated_fees() {
        let (env, admin, oracle_address, usdc_address, oracle_pk, _) = setup_test_env();
        let contract_id = env.register(TimeLockContract, ());

        env.as_contract(&contract_id, || {
            TimeLockContract::initialize(
                env.clone(),
                admin.clone(),
                oracle_pk.clone(),
                oracle_address,
                usdc_address,
            ).unwrap();
        });

        let sender = <soroban_sdk::Address as TestAddress>::generate(&env);
        let claimant = <soroban_sdk::Address as TestAddress>::generate(&env);
        let fee_recipient = <soroban_sdk::Address as TestAddress>::generate(&env);

        let current_time = 1000000u64;
        env.ledger().set_timestamp(current_time);

        let unlock_time = current_time + 3600;
        let amount = 10000000i128;
        let phone_hash = String::from_str(&env, "+1234567890");

        // Create and withdraw gift to accumulate fees
        let gift_id = env.as_contract(&contract_id, || {
            TimeLockContract::create_gift(
                env.clone(),
                sender.clone(),
                amount,
                unlock_time,
                phone_hash.clone(),
            ).unwrap()
        });

        env.ledger().set_timestamp(unlock_time);
        let verification_proof = BytesN::from_array(&env, &[0u8; 64]);

        env.as_contract(&contract_id, || {
            TimeLockContract::claim_gift(env.clone(), claimant.clone(), gift_id, verification_proof).unwrap();
            TimeLockContract::withdraw_gift(env.clone(), gift_id).unwrap();
        });

        // Check accumulated fees
        let expected_fee = (amount * constants::GIFT_FEE_BPS as i128) / 10000;
        let total_fees = env.as_contract(&contract_id, || {
            TimeLockContract::get_total_fees(env.clone()).unwrap()
        });
        assert_eq!(total_fees, expected_fee);

        // Admin withdraws fees
        env.as_contract(&contract_id, || {
            TimeLockContract::withdraw_accumulated_fees(env.clone(), fee_recipient.clone()).unwrap()
        });

        // Verify fees are reset
        let total_fees_after = env.as_contract(&contract_id, || {
            TimeLockContract::get_total_fees(env.clone()).unwrap()
        });
        assert_eq!(total_fees_after, 0);

        // Verify event was emitted
        let events = env.events().all();
        let fee_withdrawal_event = events.iter().find(|e| {
            e.topics.len() > 0 && e.topics.get(0).unwrap() == &soroban_sdk::symbol_short!("fee_wdr")
        });
        assert!(fee_withdrawal_event.is_some());
    }

    #[test]
    fn test_non_admin_cannot_withdraw_fees() {
        let (env, admin, oracle_address, usdc_address, oracle_pk, _) = setup_test_env();
        let contract_id = env.register(TimeLockContract, ());

        env.as_contract(&contract_id, || {
            TimeLockContract::initialize(
                env.clone(),
                admin,
                oracle_pk,
                oracle_address,
                usdc_address,
            ).unwrap();
        });

        let non_admin = <soroban_sdk::Address as TestAddress>::generate(&env);
        let fee_recipient = <soroban_sdk::Address as TestAddress>::generate(&env);

        // Non-admin tries to withdraw fees - should fail
        let result = env.as_contract(&contract_id, || {
            TimeLockContract::withdraw_accumulated_fees(env.clone(), fee_recipient)
        });
        assert_eq!(result, Err(Error::Unauthorized));
    }
}
