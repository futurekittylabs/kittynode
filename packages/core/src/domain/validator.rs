use serde::{Deserialize, Serialize};

/// Represents a validator key pair.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatorKey {
    pub public_key: String,
    pub secret_key: String,
}

/// Deposit data payload used for validator activation on the beacon chain.
///
/// The structure mirrors the Ethereum staking launchpad `deposit-cli` output so
/// that generated files can be uploaded without additional transformation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepositData {
    pub pubkey: String,
    pub withdrawal_credentials: String,
    pub amount: u64,
    pub signature: String,
    pub deposit_message_root: String,
    pub deposit_data_root: String,
    pub fork_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_name: Option<String>,
}
