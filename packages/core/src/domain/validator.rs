use serde::{Deserialize, Serialize};

/// Represents a validator key pair.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatorKey {
    pub public_key: String,
    pub secret_key: String,
}

/// Deposit data payload used for validator activation on the beacon chain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepositData {
    pub public_key: String,
    pub withdrawal_credentials: String,
    pub amount_gwei: u64,
    pub signature: String,
    pub deposit_message_root: String,
    pub deposit_data_root: String,
    pub fork_version: String,
}
