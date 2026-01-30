use crate::constants;
use crate::errors::Error;
use crate::events::{
    AnchorDepositSent, BankWithdrawalInitiated, FeesCollected, OracleAddressUpdated,
    OracleRateQueried, PathPaymentExecuted, SlippageConfigUpdated, WithdrawalSuccess, FeeWithdrawal,
};
use crate::oracle::{self, OracleConfig};
use crate::path_payment;
use crate::slippage::{self, SlippageConfig};
use crate::storage;
use crate::token;
use crate::types::{Gift, GiftStatus};
use soroban_sdk::{
    contract, contractimpl, symbol_short, xdr::ToXdr, Address, Bytes, BytesN, Env, String,
};

#[contract]
pub struct TimeLockContract;

use crate::interface::TimeLockTrait;

#[contractimpl]
impl TimeLockTrait for TimeLockContract {
    /// Initialize contract with admin, oracle auth key (for claims), and oracle config (for price feed)
    fn initialize(
        env: Env,
        admin: Address,
        oracle_pk: BytesN<32>,
        oracle_address: Address,
        usdc_address: Address,
    ) -> Result<(), Error> {
        if storage::has_admin(&env) {
            return Err(Error::Unauthorized);
        }

        storage::set_admin(&env, &admin);
        storage::set_oracle_auth_key(&env, &oracle_pk);

        let oracle_config = oracle::default_oracle_config(oracle_address);
        storage::set_oracle_config(&env, &oracle_config);

        let slippage_config = slippage::default_slippage_config(admin);
        storage::set_slippage_config(&env, &slippage_config);

        storage::set_usdc_address(&env, &usdc_address);

        Ok(())
    }

    fn create_gift(
        env: Env,
        sender: Address,
        amount: i128,
        unlock_timestamp: u64,
        recipient_phone_hash: String,
    ) -> Result<u64, Error> {
        sender.require_auth();

        if amount < constants::MIN_GIFT_AMOUNT || amount > constants::MAX_GIFT_AMOUNT {
            return Err(Error::InvalidAmount);
        }

        let gift_id = storage::increment_next_gift_id(&env);

        let gift = Gift {
            sender: sender.clone(),
            recipient: None,
            amount,
            unlock_timestamp,
            recipient_phone_hash,
            status: GiftStatus::Created,
        };

        storage::set_gift(&env, gift_id, &gift);

        // Internal Tracking: Transfer USDC from sender to contract
        let usdc_address = storage::get_usdc_address(&env).ok_or(Error::InvalidTokenAddress)?;
        token::transfer_from(&env, &usdc_address, &sender, &env.current_contract_address(), amount)?;

        // Update internal accounting
        let total_held = storage::get_total_held(&env) + amount;
        let total_gifted = storage::get_total_gifted(&env) + amount;
        storage::set_total_held(&env, total_held);
        storage::set_total_gifted(&env, total_gifted);

        Ok(gift_id)
    }

    fn claim_gift(
        env: Env,
        claimant: Address,
        gift_id: u64,
        verification_proof: BytesN<64>,
    ) -> Result<(), Error> {
        claimant.require_auth();

        let mut gift = storage::get_gift(&env, gift_id).ok_or(Error::GiftNotFound)?;

        if gift.status != GiftStatus::Created {
            return Err(if gift.status == GiftStatus::Claimed {
                Error::AlreadyClaimed
            } else {
                Error::InvalidStatus
            });
        }

        if env.ledger().timestamp() < gift.unlock_timestamp {
            return Err(Error::NotUnlocked);
        }

        let oracle_pk = storage::get_oracle_auth_key(&env);

        let mut payload = Bytes::new(&env);
        payload.append(&claimant.clone().to_xdr(&env));
        payload.append(&gift.recipient_phone_hash.clone().to_xdr(&env));

        env.crypto()
            .ed25519_verify(&oracle_pk, &payload, &verification_proof);

        gift.recipient = Some(claimant.clone());
        gift.status = GiftStatus::Unlocked;

        storage::set_gift(&env, gift_id, &gift);

        env.events().publish(
            (symbol_short!("claimed"),),
            (gift_id, claimant, env.ledger().timestamp()),
        );

        Ok(())
    }

    fn withdraw_to_bank(
        env: Env,
        gift_id: u64,
        recipient_bank_details_hash: String,
        anchor_address: Address,
    ) -> Result<(), Error> {
        let mut gift = storage::get_gift(&env, gift_id).ok_or(Error::GiftNotFound)?;

        if gift.status != GiftStatus::Unlocked {
            return Err(Error::InvalidStatus);
        }

        let recipient = gift.recipient.as_ref().ok_or(Error::Unauthorized)?;
        recipient.require_auth();

        let fee_amount = (gift.amount * constants::GIFT_FEE_BPS as i128) / 10000;
        let amount_after_fee = gift.amount - fee_amount;

        env.events().publish(
            (symbol_short!("fee_coll"),),
            FeesCollected {
                gift_id,
                fee_amount_usdc: fee_amount,
            },
        );

        let oracle_config = storage::get_oracle_config(&env).ok_or(Error::OracleUnavailable)?;
        let oracle_rate = oracle::get_price(&env, &oracle_config)?;
        let expected_ngn = (amount_after_fee * oracle_rate) / 1_000_000;

        env.events().publish(
            (symbol_short!("bank_init"),),
            BankWithdrawalInitiated {
                gift_id,
                amount_usdc: amount_after_fee,
                expected_ngn,
            },
        );

        let slippage_config = storage::get_slippage_config(&env).ok_or(Error::Unauthorized)?;
        let min_ngn = (expected_ngn * (10000 - slippage_config.max_slippage_bps as i128)) / 10000;

        let path = path_payment::discover_optimal_path(
            &env,
            &recipient.clone(),
            &anchor_address,
            amount_after_fee,
        )?;

        let swap_result = path_payment::execute_path_payment(
            &env,
            amount_after_fee,
            min_ngn,
            path.clone(),
            &anchor_address,
        )?;

        env.events().publish(
            (symbol_short!("path_exec"),),
            PathPaymentExecuted {
                usdc_sent: swap_result.amount_in,
                ngn_received: swap_result.amount_out,
                exchange_rate: (swap_result.amount_out * 1_000_000) / swap_result.amount_in,
                path,
            },
        );

        env.events().publish(
            (symbol_short!("anc_dep"),),
            AnchorDepositSent {
                anchor_address,
                ngn_amount: swap_result.amount_out,
                memo: recipient_bank_details_hash,
            },
        );

        gift.status = GiftStatus::Withdrawn;
        storage::set_gift(&env, gift_id, &gift);

        // Internal Tracking: Collect Platform Fee
        let total_fees = storage::get_total_fees(&env) + fee_amount;
        storage::set_total_fees(&env, total_fees);

        // Note: total_held doesn't change yet as the contract still holds the USDC
        // until it's actually swapped/sent.
        // In this implementation, withdraw_to_bank simulates the swap.
        // For strict tracking, we decrease total_held by the full gift amount.
        let total_held = storage::get_total_held(&env) - gift.amount;
        storage::set_total_held(&env, total_held);

        Ok(())
    }

    fn withdraw_gift(
        env: Env,
        gift_id: u64,
    ) -> Result<(), Error> {
        let mut gift = storage::get_gift(&env, gift_id).ok_or(Error::GiftNotFound)?;

        if gift.status != GiftStatus::Unlocked {
            return Err(Error::InvalidStatus);
        }

        let recipient = gift.recipient.as_ref().ok_or(Error::Unauthorized)?;
        recipient.require_auth();

        let fee_amount = (gift.amount * constants::GIFT_FEE_BPS as i128) / 10000;
        let amount_after_fee = gift.amount - fee_amount;

        // Internal Tracking: Collect Platform Fee
        let total_fees = storage::get_total_fees(&env) + fee_amount;
        storage::set_total_fees(&env, total_fees);

        // Update internal accounting: Decrease total_held by the full gift amount
        let total_held = storage::get_total_held(&env) - gift.amount;
        storage::set_total_held(&env, total_held);

        // Events
        env.events().publish(
            (symbol_short!("fee_coll"),),
            FeesCollected {
                gift_id,
                fee_amount_usdc: fee_amount,
            },
        );

        env.events().publish(
            (symbol_short!("withdr_s"),),
            WithdrawalSuccess {
                gift_id,
                recipient: recipient.clone(),
                amount_withdrawn: amount_after_fee,
                timestamp: env.ledger().timestamp(),
            },
        );

        // Transfer USDC to recipient
        let usdc_address = storage::get_usdc_address(&env).ok_or(Error::InvalidTokenAddress)?;
        token::transfer(&env, &usdc_address, recipient, amount_after_fee)?;

        gift.status = GiftStatus::Withdrawn;
        storage::set_gift(&env, gift_id, &gift);

        Ok(())
    }

    fn withdraw_accumulated_fees(
        env: Env,
        to: Address,
    ) -> Result<(), Error> {
        let admin = storage::get_admin(&env).ok_or(Error::Unauthorized)?;
        admin.require_auth();

        let total_fees = storage::get_total_fees(&env);
        if total_fees == 0 {
             return Ok(());
        }

        let usdc_address = storage::get_usdc_address(&env).ok_or(Error::InvalidTokenAddress)?;
        token::transfer(&env, &usdc_address, &to, total_fees)?;

        storage::set_total_fees(&env, 0);

         env.events().publish(
            (symbol_short!("fee_wdr"),),
            FeeWithdrawal {
                total_fees,
                to,
            },
        );

        Ok(())
    }

    fn set_oracle_address(env: Env, new_oracle_address: Address) -> Result<(), Error> {
        let admin = storage::get_admin(&env).ok_or(Error::Unauthorized)?;
        admin.require_auth();

        let mut oracle_config = storage::get_oracle_config(&env).ok_or(Error::OracleUnavailable)?;
        let old_address = oracle_config.oracle_address.clone();
        oracle_config.oracle_address = new_oracle_address.clone();

        storage::set_oracle_config(&env, &oracle_config);

        env.events().publish(
            (symbol_short!("oracle_ad"),),
            OracleAddressUpdated {
                old_address,
                new_address: new_oracle_address,
            },
        );

        Ok(())
    }

    fn set_max_oracle_age(env: Env, max_age: u64) -> Result<(), Error> {
        let admin = storage::get_admin(&env).ok_or(Error::Unauthorized)?;
        admin.require_auth();

        let mut oracle_config = storage::get_oracle_config(&env).ok_or(Error::OracleUnavailable)?;
        oracle_config.max_oracle_age = max_age;

        storage::set_oracle_config(&env, &oracle_config);

        Ok(())
    }

    fn set_oracle_paused(env: Env, paused: bool) -> Result<(), Error> {
        let admin = storage::get_admin(&env).ok_or(Error::Unauthorized)?;
        admin.require_auth();

        let mut oracle_config = storage::get_oracle_config(&env).ok_or(Error::OracleUnavailable)?;
        oracle_config.is_paused = paused;

        storage::set_oracle_config(&env, &oracle_config);

        Ok(())
    }

    fn set_max_slippage(env: Env, slippage_bps: u32) -> Result<(), Error> {
        slippage::validate_slippage_bounds(slippage_bps)?;

        let admin = storage::get_admin(&env).ok_or(Error::Unauthorized)?;
        admin.require_auth();

        let mut slippage_config = storage::get_slippage_config(&env).ok_or(Error::Unauthorized)?;
        let old_slippage = slippage_config.max_slippage_bps;
        slippage_config.max_slippage_bps = slippage_bps;

        storage::set_slippage_config(&env, &slippage_config);

        env.events().publish(
            (symbol_short!("slip_upd"),),
            SlippageConfigUpdated {
                old_slippage,
                new_slippage: slippage_bps,
                admin,
            },
        );

        Ok(())
    }

    fn check_exchange_rate(env: Env, currency_pair: String) -> Result<i128, Error> {
        let oracle_config = storage::get_oracle_config(&env).ok_or(Error::OracleUnavailable)?;
        let oracle_rate = oracle::get_price(&env, &oracle_config)?;

        env.events().publish(
            (symbol_short!("price_q"),),
            OracleRateQueried {
                timestamp: env.ledger().timestamp(),
                rate: oracle_rate,
                source: currency_pair,
            },
        );

        Ok(oracle_rate)
    }

    fn validate_slippage(env: Env, oracle_rate: i128, actual_rate: i128) -> Result<(), Error> {
        let slippage_config = storage::get_slippage_config(&env).ok_or(Error::Unauthorized)?;
        slippage::validate_slippage(&env, &slippage_config, oracle_rate, actual_rate)
    }

    fn get_oracle_config(env: Env) -> Result<OracleConfig, Error> {
        storage::get_oracle_config(&env).ok_or(Error::OracleUnavailable)
    }

    fn get_slippage_config(env: Env) -> Result<SlippageConfig, Error> {
        storage::get_slippage_config(&env).ok_or(Error::Unauthorized)
    }

    fn get_gift(env: Env, gift_id: u64) -> Result<Gift, Error> {
        storage::get_gift(&env, gift_id).ok_or(Error::GiftNotFound)
    }

    fn get_balance(env: Env, owner: Address) -> Result<i128, Error> {
        let usdc_address = storage::get_usdc_address(&env).ok_or(Error::InvalidTokenAddress)?;
        Ok(token::balance_of(&env, &usdc_address, &owner))
    }

    fn get_total_held(env: Env) -> Result<i128, Error> {
        Ok(storage::get_total_held(&env))
    }

    fn get_total_fees(env: Env) -> Result<i128, Error> {
        Ok(storage::get_total_fees(&env))
    }
}
