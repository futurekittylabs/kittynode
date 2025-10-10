#[cfg(target_os = "linux")]
pub use crate::application::validator::swap_active;
pub use crate::application::validator::{
    ValidatorKeygenOutcome, ValidatorKeygenRequest, ValidatorProgress, available_networks,
    check_internet_connectivity, default_withdrawal_address, derive_execution_address,
    format_eth_from_gwei, generate_validator_files, generate_validator_files_with_progress,
    normalize_withdrawal_address, parse_deposit_amount_gwei, parse_validator_count,
    resolve_withdrawal_address, validate_password,
};
