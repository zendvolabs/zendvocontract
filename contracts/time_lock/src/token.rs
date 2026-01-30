#![allow(unused)]
use soroban_sdk::{token, Address, Env};
use crate::errors::Error;

/// Wrapper for balance_of(address)
pub fn balance_of(env: &Env, token_address: &Address, owner: &Address) -> i128 {
    let client = token::Client::new(env, token_address);
    client.balance(owner)
}

/// Wrapper for allowance(from, to)
#[allow(dead_code)]
pub fn allowance(env: &Env, token_address: &Address, from: &Address, spender: &Address) -> i128 {
    let client = token::Client::new(env, token_address);
    client.allowance(from, spender)
}

/// Wrapper for transfer(to, amount)
#[allow(dead_code)]
pub fn transfer(env: &Env, token_address: &Address, to: &Address, amount: i128) -> Result<(), Error> {
    let client = token::Client::new(env, token_address);
    
    // In Soroban SDK, transfer will panic if it fails (e.g. insufficient funds)
    // However, we can check balances first or catch errors if using try_
    client.transfer(&env.current_contract_address(), to, &amount);
    
    Ok(())
}

/// Wrapper for transfer_from(from, to, amount)
pub fn transfer_from(
    env: &Env,
    token_address: &Address,
    from: &Address,
    to: &Address,
    amount: i128,
) -> Result<(), Error> {
    let client = token::Client::new(env, token_address);
    
    // Check allowance first to provide better error mapping
    let current_allowance = client.allowance(from, &env.current_contract_address());
    if current_allowance < amount {
        return Err(Error::InsufficientAllowance);
    }
    
    // Check balance
    if client.balance(from) < amount {
        return Err(Error::InsufficientFunds);
    }
    
    client.transfer_from(&env.current_contract_address(), from, to, &amount);
    
    Ok(())
}
