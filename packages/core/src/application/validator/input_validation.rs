use alloy_primitives::U256;
use alloy_primitives::utils::parse_units;
use eyre::{Result, eyre};
use url::Url;

pub const MIN_VALIDATOR_COUNT: u16 = 1;
pub const MAX_VALIDATOR_COUNT: u16 = 32;

const MIN_PASSWORD_LEN: usize = 12;
const MAX_PASSWORD_LEN: usize = 128;

pub fn parse_validator_count(input: &str) -> Result<u16> {
    let trimmed = input.trim();
    let count: u16 = trimmed.parse().map_err(|_| {
        eyre!(
            "Validator count must be a whole number between {MIN_VALIDATOR_COUNT} and {MAX_VALIDATOR_COUNT}"
        )
    })?;
    if !(MIN_VALIDATOR_COUNT..=MAX_VALIDATOR_COUNT).contains(&count) {
        return Err(eyre!(
            "Validator count must be between {MIN_VALIDATOR_COUNT} and {MAX_VALIDATOR_COUNT}"
        ));
    }
    Ok(count)
}

pub fn normalize_withdrawal_address(input: &str) -> Result<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(eyre!("Withdrawal address is required"));
    }

    let body = if let Some(rest) = trimmed.strip_prefix("0x") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("0X") {
        rest
    } else {
        trimmed
    };

    if body.len() != 40 {
        return Err(eyre!(
            "Withdrawal address must be 40 hexadecimal characters (42 with 0x prefix)"
        ));
    }

    if !body.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(eyre!(
            "Withdrawal address must contain only hexadecimal characters"
        ));
    }

    Ok(format!("0x{}", body.to_ascii_lowercase()))
}

/// Parses an ETH amount into gwei using decimal string math to avoid floating point rounding.
///
/// Accepts at most 9 decimal places and returns total gwei as `u64`.
/// Range validation (e.g., 1–32 ETH per validator) is enforced by callers.
pub fn parse_deposit_amount_gwei(input: &str) -> Result<u64> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(eyre!("Deposit amount is required"));
    }
    let as_u256: U256 = parse_units(trimmed, 9)
        .map_err(|_| eyre!("Deposit amount must be a valid decimal number"))?
        .into();
    let total_gwei: u64 = as_u256
        .try_into()
        .map_err(|_| eyre!("Deposit amount exceeds supported maximum"))?;
    Ok(total_gwei)
}

pub fn validate_password(password: &str) -> Result<()> {
    let length = password.chars().count();
    if length < MIN_PASSWORD_LEN {
        return Err(eyre!(
            "Password must be at least {MIN_PASSWORD_LEN} characters long"
        ));
    }
    if length > MAX_PASSWORD_LEN {
        return Err(eyre!(
            "Password must be at most {MAX_PASSWORD_LEN} characters long"
        ));
    }
    Ok(())
}

/// Validates an Ethereum endpoint URL format
/// Accepts formats like:
/// - http://localhost:8545
/// - http://192.168.1.100:5052
/// - http://25.67.109.175:8545
pub fn validate_endpoint_url(endpoint: &str) -> Result<()> {
    let trimmed = endpoint.trim();
    if trimmed.is_empty() {
        return Err(eyre!("Endpoint URL cannot be empty"));
    }

    // Try to parse as URL
    let url = Url::parse(trimmed).map_err(|e| eyre!("Invalid endpoint URL format: {e}"))?;

    // Check scheme
    match url.scheme() {
        "http" | "https" => {}
        other => {
            return Err(eyre!(
                "Endpoint must use http or https scheme, got: {other}"
            ));
        }
    }

    // Check host exists
    if url.host_str().is_none() {
        return Err(eyre!("Endpoint URL must include a host"));
    }

    // Check port exists
    if url.port().is_none() {
        return Err(eyre!(
            "Endpoint URL must include a port (e.g., :8545 or :5052)"
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validator_count_within_bounds() {
        assert_eq!(parse_validator_count("1").unwrap(), 1);
        assert_eq!(parse_validator_count("32").unwrap(), 32);
    }

    #[test]
    fn validator_count_below_min_errors() {
        assert!(parse_validator_count("0").is_err());
    }

    #[test]
    fn validator_count_above_max_errors() {
        assert!(parse_validator_count("33").is_err());
    }

    #[test]
    fn validator_count_non_numeric_errors() {
        assert!(parse_validator_count("abc").is_err());
    }

    #[test]
    fn normalize_withdrawal_address_accepts_prefixed_hex() {
        let normalized =
            normalize_withdrawal_address("0xABCDEFabcdefABCDEFabcdefABCDEFabcdefABCD").unwrap();
        assert_eq!(normalized, "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd");
    }

    #[test]
    fn normalize_withdrawal_address_accepts_unprefixed_hex() {
        let normalized =
            normalize_withdrawal_address("ABCDEFabcdefABCDEFabcdefABCDEFabcdefABCD").unwrap();
        assert_eq!(normalized, "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd");
    }

    #[test]
    fn normalize_withdrawal_address_rejects_invalid_length() {
        assert!(normalize_withdrawal_address("0x1234").is_err());
    }

    #[test]
    fn normalize_withdrawal_address_rejects_invalid_chars() {
        assert!(normalize_withdrawal_address("0xZZzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz").is_err());
    }

    #[test]
    fn validate_password_accepts_bounds() {
        let password = "a".repeat(12);
        assert!(validate_password(&password).is_ok());
        let long_password = "a".repeat(128);
        assert!(validate_password(&long_password).is_ok());
    }

    #[test]
    fn validate_password_rejects_short() {
        let password = "short";
        assert!(validate_password(password).is_err());
    }

    #[test]
    fn validate_password_rejects_long() {
        let password = "a".repeat(129);
        assert!(validate_password(&password).is_err());
    }
}
