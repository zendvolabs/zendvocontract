#![cfg(test)]
extern crate std;

use super::*;
use soroban_sdk::{testutils::{Address as _, Ledger}, Address, Bytes, BytesN, Env, String, xdr::ToXdr};
use ed25519_dalek::{Signer, SigningKey};
use rand::rngs::OsRng;

#[test]
fn test_claim_gift() {
    let env = Env::default();
    env.mock_all_auths();

    // 1. Initialize Oracle
    let mut csprng = OsRng;
    let oracle_keypair = SigningKey::generate(&mut csprng);
    let oracle_pub_bytes = oracle_keypair.verifying_key().to_bytes();
    let oracle_pk = BytesN::from_array(&env, &oracle_pub_bytes);

    let contract_id = env.register(TimeLockContract, ());
    let client = TimeLockContractClient::new(&env, &contract_id);

    client.initialize(&oracle_pk);

    // 2. Create Gift
    let sender = Address::generate(&env);
    let recipient_phone_hash = String::from_str(&env, "hash_of_phone_number");
    let amount = 10_000_000;
    let unlock_time = env.ledger().timestamp() + 100;

    let gift_id = client.create_gift(
        &sender,
        &amount,
        &unlock_time,
        &recipient_phone_hash,
    );

    // 3. Prepare Claim
    let claimant = Address::generate(&env);
    
    // Construct payload for signature: claimant XDR + phone hash XDR
    let mut payload = Bytes::new(&env);
    payload.append(&claimant.clone().to_xdr(&env));
    payload.append(&recipient_phone_hash.clone().to_xdr(&env));
    
    // Sign payload
    let len = payload.len() as usize;
    let mut payload_vec = std::vec![0u8; len];
    payload.copy_into_slice(&mut payload_vec);
    
    let signature = oracle_keypair.sign(&payload_vec);
    let signature_bytes = signature.to_bytes();
    let proof = BytesN::from_array(&env, &signature_bytes);

    // 4. Try to claim early (should fail)
    let res = client.try_claim_gift(&claimant, &gift_id, &proof);
    assert!(res.is_err());
    assert_eq!(res.err(), Some(Ok(Error::NotUnlocked)));

    // 5. Advance time
    env.ledger().set_timestamp(unlock_time + 1);

    // 6. Claim successfully
    let res = client.try_claim_gift(&claimant, &gift_id, &proof);
    assert!(res.is_ok());

    // 7. Try to claim again (should fail)
    let res = client.try_claim_gift(&claimant, &gift_id, &proof);
    assert_eq!(res.err(), Some(Ok(Error::AlreadyClaimed)));
}

#[test]
#[should_panic]
fn test_invalid_proof() {
    let env = Env::default();
    env.mock_all_auths();
    
    let mut csprng = OsRng;
    let oracle_keypair = SigningKey::generate(&mut csprng);
    let oracle_pk = BytesN::from_array(&env, &oracle_keypair.verifying_key().to_bytes());

    let contract_id = env.register(TimeLockContract, ());
    let client = TimeLockContractClient::new(&env, &contract_id);

    client.initialize(&oracle_pk);

    let sender = Address::generate(&env);
    let recipient_phone_hash = String::from_str(&env, "hash_of_phone_number");
    let amount = 10_000_000;
    let unlock_time = env.ledger().timestamp(); 

    let gift_id = client.create_gift(
        &sender,
        &amount,
        &unlock_time,
        &recipient_phone_hash,
    );

    let claimant = Address::generate(&env);
    
    // Wrong payload (different phone hash)
    let wrong_hash = String::from_str(&env, "wrong_hash");
    let mut payload = Bytes::new(&env);
    payload.append(&claimant.clone().to_xdr(&env));
    payload.append(&wrong_hash.clone().to_xdr(&env));
    
    let mut payload_vec = std::vec![0u8; payload.len() as usize];
    payload.copy_into_slice(&mut payload_vec);
    
    let signature = oracle_keypair.sign(&payload_vec);
    let proof = BytesN::from_array(&env, &signature.to_bytes());

    // Should panic because of crypto verification failure
    client.claim_gift(&claimant, &gift_id, &proof);
mod tests {
    use crate::*;

    #[test]
    fn test_calculate_rate_difference() {
        let diff = slippage::calculate_rate_difference(1000000, 1010000);
        assert_eq!(diff, 100);

        let diff = slippage::calculate_rate_difference(1000000, 1050000);
        assert_eq!(diff, 500);

        let diff = slippage::calculate_rate_difference(1000000, 990000);
        assert_eq!(diff, -100);
    }

    #[test]
    fn test_calculate_expected_output() {
        let output = slippage::calculate_expected_output(1000000, 1000, 200);
        assert_eq!(output, 980);
    }

    #[test]
    fn test_validate_rate_bounds() {
        assert!(oracle::validate_rate_bounds(1000000).is_ok());
        assert!(oracle::validate_rate_bounds(0).is_err());
        assert!(oracle::validate_rate_bounds(-1000000).is_err());
    }

    #[test]
    fn test_validate_slippage_bounds() {
        assert!(slippage::validate_slippage_bounds(200).is_ok());
        assert!(slippage::validate_slippage_bounds(10000).is_ok());
        assert!(slippage::validate_slippage_bounds(10001).is_err());
    }

    #[test]
    fn test_rate_difference_calculations() {
        // Test various rate differences
        assert_eq!(slippage::calculate_rate_difference(1000000, 1000000), 0);
        assert_eq!(slippage::calculate_rate_difference(1000000, 1100000), 1000); // 10%
        assert_eq!(slippage::calculate_rate_difference(2000000, 2200000), 1000); // 10%
        assert_eq!(slippage::calculate_rate_difference(500000, 450000), -1000); // -10%
    }

    #[test]
    fn test_expected_output_calculations() {
        // Base case: 1000 units * 1.0 rate = 1000 with 2% slippage = 980
        assert_eq!(slippage::calculate_expected_output(1000000, 1000, 200), 980);

        // No slippage
        assert_eq!(slippage::calculate_expected_output(1000000, 1000, 0), 1000);

        // Max slippage 10%
        let output = slippage::calculate_expected_output(1000000, 1000, 10000);
        assert_eq!(output, 0); // 1000 - (1000 * 10000 / 10000) = 0

        // Different exchange rate
        assert_eq!(slippage::calculate_expected_output(2000000, 500, 200), 980);
        // Base: (500 * 2000000) / 1000000 = 1000
        // Slippage: (1000 * 200) / 10000 = 20
        // Result: 1000 - 20 = 980
    }
}
