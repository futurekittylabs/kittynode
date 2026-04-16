#[path = "validator/deposit.rs"]
mod deposit;
#[path = "validator/input_validation.rs"]
mod input_validation;
#[path = "validator/keygen.rs"]
mod keygen;

pub use crate::ethereum::{
    EPHEMERY_CHECKPOINT_URLS, EPHEMERY_NETWORK_NAME, EphemeryConfig, ensure_ephemery_config,
};
pub use input_validation::{
    normalize_withdrawal_address, parse_deposit_amount_gwei, parse_validator_count,
    validate_endpoint_url, validate_password,
};
#[cfg(target_os = "linux")]
pub use keygen::swap_active;
pub use keygen::{
    ValidatorKeygenOutcome, ValidatorKeygenRequest, ValidatorProgress, available_networks,
    check_internet_connectivity, default_withdrawal_address, derive_execution_address,
    format_eth_from_gwei, generate_validator_files, generate_validator_files_with_progress,
    resolve_withdrawal_address,
};
