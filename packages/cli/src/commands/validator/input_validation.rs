use alloy_primitives::U256;
use alloy_primitives::utils::parse_units;
use eyre::{Result, eyre};

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

pub fn parse_deposit_amount(input: &str) -> Result<f64> {
    let trimmed = input.trim();
    let amount: f64 = trimmed
        .parse()
        .map_err(|_| eyre!("Deposit amount must be a valid decimal number"))?;
    if !amount.is_finite() {
        return Err(eyre!("Deposit amount must be a finite number"));
    }
    Ok(amount)
}

/// Parses an ETH amount into gwei using decimal string math to avoid floating point rounding.
///
/// Accepts at most 9 decimal places. Returns total gwei as `u64` and validates the
/// amount is within the configured min/max ETH bounds.
pub fn parse_deposit_amount_gwei(input: &str) -> Result<u64> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(eyre!("Deposit amount is required"));
    }
    // Use battle-tested parsing from alloy to convert decimal ETH to gwei (9 decimals).
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
    fn parse_deposit_amount_accepts_decimal() {
        let amount = parse_deposit_amount("32.5").unwrap();
        assert!((amount - 32.5).abs() < f64::EPSILON);
    }

    #[test]
    fn parse_deposit_amount_rejects_non_numeric() {
        assert!(parse_deposit_amount("abc").is_err());
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
