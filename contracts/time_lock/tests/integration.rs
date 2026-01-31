#![cfg(test)]
extern crate std;

use soroban_sdk::{testutils::{Address as _, Ledger}, Address, Bytes, BytesN, Env, String, xdr::ToXdr};
use ed25519_dalek::{Signer, SigningKey};
use rand::rngs::OsRng;
use zendvo_time_lock::{TimeLockContract, TimeLockContractClient};

#[test]
fn test_claim_gift() {
    let env = Env::default();
    env.mock_all_auths();

    let mut csprng = OsRng;
    let oracle_keypair = SigningKey::generate(&mut csprng);
    let oracle_pub_bytes = oracle_keypair.verifying_key().to_bytes();
    let oracle_pk = BytesN::from_array(&env, &oracle_pub_bytes);

    let contract_id = env.register(TimeLockContract, ());
    let client = TimeLockContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let oracle_address = Address::generate(&env);
    
    // Mock USDC Token
    let usdc_admin = Address::generate(&env);
    let usdc_address = env.register_stellar_asset_contract_v2(usdc_admin.clone()).address();
    
    client.initialize(&admin, &oracle_pk, &oracle_address, &usdc_address);

    let sender = Address::generate(&env);
    let recipient_phone_hash = BytesN::from_array(&env, &[10u8; 32]);
    let amount = 10_000_000;
    
    // Distribute USDC to sender and approve contract
    let usdc_client = soroban_sdk::token::StellarAssetClient::new(&env, &usdc_address);
    usdc_client.mint(&sender, &amount);
    
    let usdc_token = soroban_sdk::token::Client::new(&env, &usdc_address);
    usdc_token.approve(&sender, &contract_id, &amount, &(env.ledger().sequence() + 100));

    let unlock_time = env.ledger().timestamp() + 100;
    let gift_id = client.create_gift(&sender, &amount, &unlock_time, &recipient_phone_hash);

    let claimant = Address::generate(&env);
    let mut payload = Bytes::new(&env);
    payload.append(&claimant.clone().to_xdr(&env));
    payload.append(&recipient_phone_hash.clone().to_xdr(&env));
    
    let mut payload_vec = std::vec![0u8; payload.len() as usize];
    payload.copy_into_slice(&mut payload_vec);
    let signature = oracle_keypair.sign(&payload_vec);
    let proof = BytesN::from_array(&env, &signature.to_bytes());

    let res = client.try_claim_gift(&claimant, &gift_id, &proof);
    assert!(res.is_err()); // Early claim

    env.ledger().set_timestamp(unlock_time + 1);
    let res = client.try_claim_gift(&claimant, &gift_id, &proof);
    assert!(res.is_ok());
}

#[test]
fn test_withdraw_to_bank_success() {
    let env = Env::default();
    env.mock_all_auths();

    let mut csprng = OsRng;
    let oracle_keypair = SigningKey::generate(&mut csprng);
    let oracle_pk = BytesN::from_array(&env, &oracle_keypair.verifying_key().to_bytes());

    let contract_id = env.register(TimeLockContract, ());
    let client = TimeLockContractClient::new(&env, &contract_id);
    
    let usdc_address = env.register_stellar_asset_contract_v2(Address::generate(&env)).address();
    client.initialize(&Address::generate(&env), &oracle_pk, &Address::generate(&env), &usdc_address);

    let sender = Address::generate(&env);
    let amount = 10_000_000;
    soroban_sdk::token::StellarAssetClient::new(&env, &usdc_address).mint(&sender, &amount);
    soroban_sdk::token::Client::new(&env, &usdc_address).approve(&sender, &contract_id, &amount, &(env.ledger().sequence() + 100));

    let recipient_phone_hash = BytesN::from_array(&env, &[11u8; 32]);
    let unlock_time = env.ledger().timestamp() + 100;
    let gift_id = client.create_gift(&sender, &amount, &unlock_time, &recipient_phone_hash);

    let claimant = Address::generate(&env);
    let mut payload = Bytes::new(&env);
    payload.append(&claimant.clone().to_xdr(&env));
    payload.append(&recipient_phone_hash.clone().to_xdr(&env));
    let mut payload_vec = std::vec![0u8; payload.len() as usize];
    payload.copy_into_slice(&mut payload_vec);
    let proof = BytesN::from_array(&env, &oracle_keypair.sign(&payload_vec).to_bytes());

    env.ledger().set_timestamp(unlock_time + 1);
    client.claim_gift(&claimant, &gift_id, &proof);

    let res = client.try_withdraw_to_bank(&gift_id, &String::from_str(&env, "memo"), &Address::generate(&env));
    assert!(res.is_ok());

    // Verify internal tracking
    assert_eq!(client.get_total_held(), 0);
    assert!(client.get_total_fees() > 0);
}

#[test]
fn test_withdraw_to_bank_slippage_fail() {
    let env = Env::default();
    env.mock_all_auths();

    let mut csprng = OsRng;
    let oracle_keypair = SigningKey::generate(&mut csprng);
    let oracle_pk = BytesN::from_array(&env, &oracle_keypair.verifying_key().to_bytes());
    let contract_id = env.register(TimeLockContract, ());
    let client = TimeLockContractClient::new(&env, &contract_id);
    
    let usdc_address = env.register_stellar_asset_contract_v2(Address::generate(&env)).address();
    client.initialize(&Address::generate(&env), &oracle_pk, &Address::generate(&env), &usdc_address);

    client.set_max_slippage(&50);

    let sender = Address::generate(&env);
    let amount = 10_000_000;
    soroban_sdk::token::StellarAssetClient::new(&env, &usdc_address).mint(&sender, &amount);
    soroban_sdk::token::Client::new(&env, &usdc_address).approve(&sender, &contract_id, &amount, &(env.ledger().sequence() + 100));

    let recipient_phone_hash = BytesN::from_array(&env, &[12u8; 32]);
    let unlock_time = env.ledger().timestamp() + 100;
    let gift_id = client.create_gift(&sender, &amount, &unlock_time, &recipient_phone_hash);
    
    let claimant = Address::generate(&env);
    let mut payload = Bytes::new(&env);
    payload.append(&claimant.clone().to_xdr(&env));
    payload.append(&recipient_phone_hash.clone().to_xdr(&env));
    let mut payload_vec = std::vec![0u8; payload.len() as usize];
    payload.copy_into_slice(&mut payload_vec);
    let proof = BytesN::from_array(&env, &oracle_keypair.sign(&payload_vec).to_bytes());
    env.ledger().set_timestamp(unlock_time + 1);
    client.claim_gift(&claimant, &gift_id, &proof);

    let res = client.try_withdraw_to_bank(&gift_id, &String::from_str(&env, "memo"), &Address::generate(&env));
    assert!(res.is_err());
}

#[test]
fn test_withdraw_to_bank_insufficient_liquidity() {
    let env = Env::default();
    env.mock_all_auths();

    let mut csprng = OsRng;
    let oracle_keypair = SigningKey::generate(&mut csprng);
    let oracle_pk = BytesN::from_array(&env, &oracle_keypair.verifying_key().to_bytes());
    let contract_id = env.register(TimeLockContract, ());
    let client = TimeLockContractClient::new(&env, &contract_id);
    
    let usdc_address = env.register_stellar_asset_contract_v2(Address::generate(&env)).address();
    client.initialize(&Address::generate(&env), &oracle_pk, &Address::generate(&env), &usdc_address);

    let sender = Address::generate(&env);
    let amount = 200_000_000; 
    soroban_sdk::token::StellarAssetClient::new(&env, &usdc_address).mint(&sender, &amount);
    soroban_sdk::token::Client::new(&env, &usdc_address).approve(&sender, &contract_id, &amount, &(env.ledger().sequence() + 100));

    let recipient_phone_hash = BytesN::from_array(&env, &[13u8; 32]);
    let unlock_time = env.ledger().timestamp() + 100;
    let gift_id = client.create_gift(&sender, &amount, &unlock_time, &recipient_phone_hash);

    let claimant = Address::generate(&env);
    let mut payload = Bytes::new(&env);
    payload.append(&claimant.clone().to_xdr(&env));
    payload.append(&recipient_phone_hash.clone().to_xdr(&env));
    let mut payload_vec = std::vec![0u8; payload.len() as usize];
    payload.copy_into_slice(&mut payload_vec);
    let proof = BytesN::from_array(&env, &oracle_keypair.sign(&payload_vec).to_bytes());
    env.ledger().set_timestamp(unlock_time + 1);
    client.claim_gift(&claimant, &gift_id, &proof);

    let res = client.try_withdraw_to_bank(&gift_id, &String::from_str(&env, "memo"), &Address::generate(&env));
    assert!(res.is_err());
}

#[test]
fn test_withdraw_to_bank_invalid_status() {
    let env = Env::default();
    env.mock_all_auths();

    let oracle_pk = BytesN::from_array(&env, &[0u8; 32]);
    let contract_id = env.register(TimeLockContract, ());
    let client = TimeLockContractClient::new(&env, &contract_id);
    
    let usdc_address = env.register_stellar_asset_contract_v2(Address::generate(&env)).address();
    client.initialize(&Address::generate(&env), &oracle_pk, &Address::generate(&env), &usdc_address);

    let sender = Address::generate(&env);
    let amount = 10_000_000;
    soroban_sdk::token::StellarAssetClient::new(&env, &usdc_address).mint(&sender, &amount);
    soroban_sdk::token::Client::new(&env, &usdc_address).approve(&sender, &contract_id, &amount, &(env.ledger().sequence() + 100));

    let recipient_phone_hash = BytesN::from_array(&env, &[14u8; 32]);
    let unlock_time = env.ledger().timestamp() + 100;
    let gift_id = client.create_gift(&sender, &amount, &unlock_time, &recipient_phone_hash);
    let res = client.try_withdraw_to_bank(&gift_id, &String::from_str(&env, "h"), &Address::generate(&env));
    assert!(res.is_err());
}
