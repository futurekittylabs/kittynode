use eyre::{Result, eyre};

const MIN_VALIDATOR_COUNT: u16 = 1;
const MAX_VALIDATOR_COUNT: u16 = 1024;
const MIN_DEPOSIT: f64 = 1.0;
const MAX_DEPOSIT: f64 = 32768.0;
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
    let amount: f64 = trimmed.parse().map_err(|_| {
        eyre!("Deposit amount must be a number between {MIN_DEPOSIT} and {MAX_DEPOSIT}")
    })?;
    if !amount.is_finite() {
        return Err(eyre!("Deposit amount must be a finite number"));
    }
    if !(MIN_DEPOSIT..=MAX_DEPOSIT).contains(&amount) {
        return Err(eyre!(
            "Deposit amount must be between {MIN_DEPOSIT} and {MAX_DEPOSIT} ETH"
        ));
    }
    Ok(amount)
}

pub fn parse_deposit_amount_to_gwei(input: &str) -> Result<u64> {
    let _ = parse_deposit_amount(input)?;

    let trimmed = input.trim();
    let (mantissa, exponent) = split_exponent(trimmed);

    let mut digits = String::with_capacity(mantissa.len());
    let mut fractional_len = 0usize;
    let mut saw_decimal = false;

    for (index, ch) in mantissa.chars().enumerate() {
        match ch {
            '0'..='9' => {
                digits.push(ch);
                if saw_decimal {
                    fractional_len += 1;
                }
            }
            '.' if !saw_decimal => {
                saw_decimal = true;
            }
            '+' if index == 0 => {}
            _ => return Err(eyre!("Deposit amount must be a valid decimal value")),
        }
    }

    // Remove redundant trailing zeros from the fractional component so inputs like
    // "32.0000000000" are treated as valid whole-number amounts.
    while fractional_len > 0 && digits.ends_with('0') {
        digits.pop();
        fractional_len -= 1;
    }

    if digits.is_empty() {
        digits.push('0');
    }

    let exponent = exponent
        .map(|value| {
            value
                .parse::<i32>()
                .map_err(|_| eyre!("Invalid exponent value"))
        })
        .transpose()?;

    let digits_value = digits
        .parse::<u128>()
        .map_err(|_| eyre!("Deposit amount is too large"))?;

    let power = exponent
        .unwrap_or(0)
        .checked_sub(fractional_len as i32)
        .ok_or_else(|| eyre!("Deposit amount must be a valid decimal value"))?
        + 9;

    if power < 0 {
        return Err(eyre!("Deposit amount must be specified to at least 1 gwei"));
    }

    let multiplier = 10u128
        .checked_pow(power as u32)
        .ok_or_else(|| eyre!("Deposit amount is too large"))?;

    let gwei = digits_value
        .checked_mul(multiplier)
        .ok_or_else(|| eyre!("Deposit amount is too large"))?;

    if gwei > u64::MAX as u128 {
        return Err(eyre!("Deposit amount is too large"));
    }

    Ok(gwei as u64)
}

fn split_exponent(value: &str) -> (&str, Option<&str>) {
    if let Some(index) = value.find(['e', 'E']) {
        let (mantissa, exponent) = value.split_at(index);
        let exponent = exponent.strip_prefix(['e', 'E']).unwrap_or(exponent);
        (mantissa, Some(exponent))
    } else {
        (value, None)
    }
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
        assert_eq!(parse_validator_count("1024").unwrap(), 1024);
    }

    #[test]
    fn validator_count_below_min_errors() {
        assert!(parse_validator_count("0").is_err());
    }

    #[test]
    fn validator_count_above_max_errors() {
        assert!(parse_validator_count("1025").is_err());
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
    fn parse_deposit_amount_rejects_small() {
        assert!(parse_deposit_amount("0.5").is_err());
    }

    #[test]
    fn parse_deposit_amount_rejects_large() {
        assert!(parse_deposit_amount("32768.1").is_err());
    }

    #[test]
    fn parse_deposit_amount_rejects_non_numeric() {
        assert!(parse_deposit_amount("abc").is_err());
    }

    #[test]
    fn parse_deposit_amount_to_gwei_handles_integers() {
        assert_eq!(parse_deposit_amount_to_gwei("32").unwrap(), 32_000_000_000);
    }

    #[test]
    fn parse_deposit_amount_to_gwei_handles_decimals() {
        assert_eq!(parse_deposit_amount_to_gwei("1.5").unwrap(), 1_500_000_000);
    }

    #[test]
    fn parse_deposit_amount_to_gwei_handles_exponents() {
        assert_eq!(
            parse_deposit_amount_to_gwei("1e2").unwrap(),
            100_000_000_000
        );
        assert_eq!(
            parse_deposit_amount_to_gwei("1.25e2").unwrap(),
            125_000_000_000
        );
    }

    #[test]
    fn parse_deposit_amount_to_gwei_rejects_fractional_gwei() {
        assert!(parse_deposit_amount_to_gwei("1.0000000001").is_err());
    }

    #[test]
    fn parse_deposit_amount_to_gwei_respects_padding() {
        assert_eq!(
            parse_deposit_amount_to_gwei("1.000000001").unwrap(),
            1_000_000_001
        );
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
