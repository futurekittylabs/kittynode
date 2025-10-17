mod input_validation;
mod keygen;
mod runtime;

pub use input_validation::{
    normalize_withdrawal_address, parse_deposit_amount_gwei, parse_validator_count,
    validate_password,
};
#[cfg(target_os = "linux")]
pub use keygen::swap_active;
pub use keygen::{
    ValidatorKeygenOutcome, ValidatorKeygenRequest, ValidatorProgress, available_networks,
    check_internet_connectivity, default_withdrawal_address, derive_execution_address,
    format_eth_from_gwei, generate_validator_files, generate_validator_files_with_progress,
    resolve_withdrawal_address,
};
pub use runtime::get_validator_runtime_status;
