use crate::domain::validator::{DepositData, ValidatorKey};
use eyre::Result;
use serde::{Serialize, de::DeserializeOwned};
use std::path::Path;

/// Provides cryptographic operations required by the validator slice.
pub trait CryptoProvider: Send + Sync {
    fn generate_key(&self, entropy: &str) -> Result<ValidatorKey>;
    fn create_deposit_data(
        &self,
        key: &ValidatorKey,
        withdrawal_credentials: &str,
        amount_gwei: u64,
        fork_version: [u8; 4],
        genesis_validators_root: &str,
    ) -> Result<DepositData>;
}

/// Filesystem operations needed for persisting validator artifacts.
pub trait ValidatorFilesystem: Send + Sync {
    fn ensure_secure_directory(&self, path: &Path) -> Result<()>;
    fn write_json_secure<T: Serialize + ?Sized>(
        &self,
        path: &Path,
        value: &T,
        overwrite: bool,
    ) -> Result<()>;
    fn read_json_secure<T: DeserializeOwned>(&self, path: &Path) -> Result<T>;
}
