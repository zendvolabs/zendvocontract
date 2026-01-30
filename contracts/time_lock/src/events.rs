use soroban_sdk::{contracttype, Address, String};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OracleRateQueried {
    pub timestamp: u64,
    pub rate: i128,
    pub source: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SlippageConfigUpdated {
    pub old_slippage: u32,
    pub new_slippage: u32,
    pub admin: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OracleAddressUpdated {
    pub old_address: Address,
    pub new_address: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BankWithdrawalInitiated {
    pub gift_id: u64,
    pub amount_usdc: i128,
    pub expected_ngn: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PathPaymentExecuted {
    pub usdc_sent: i128,
    pub ngn_received: i128,
    pub exchange_rate: i128,
    pub path: soroban_sdk::Vec<Address>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnchorDepositSent {
    pub anchor_address: Address,
    pub ngn_amount: i128,
    pub memo: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeesCollected {
    pub gift_id: u64,
    pub fee_amount_usdc: i128,
}
