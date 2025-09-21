use eyre::{Result, eyre};

pub(crate) fn parse_fork_version_hex(input: &str) -> Result<[u8; 4]> {
    let trimmed = input.trim().trim_start_matches("0x");
    if trimmed.len() != 8 {
        return Err(eyre!(
            "fork version must be 4 bytes (8 hex characters), received {}",
            input
        ));
    }

    let mut bytes = [0u8; 4];
    for (idx, chunk) in trimmed.as_bytes().chunks(2).enumerate() {
        let hex = std::str::from_utf8(chunk).map_err(|_| eyre!("invalid UTF-8 in fork version"))?;
        bytes[idx] =
            u8::from_str_radix(hex, 16).map_err(|_| eyre!("invalid hex in fork version: {hex}"))?;
    }
    Ok(bytes)
}

pub(crate) fn parse_genesis_validators_root_hex(input: &str) -> Result<String> {
    let trimmed = input.trim();
    let without_prefix = trimmed.strip_prefix("0x").unwrap_or(trimmed);
    if without_prefix.len() != 64 {
        return Err(eyre!(
            "genesis validators root must be 32 bytes (64 hex characters), received {}",
            input
        ));
    }

    let mut bytes = [0u8; 32];
    for (idx, chunk) in without_prefix.as_bytes().chunks(2).enumerate() {
        let hex = std::str::from_utf8(chunk)
            .map_err(|_| eyre!("invalid UTF-8 in genesis validators root"))?;
        bytes[idx] = u8::from_str_radix(hex, 16)
            .map_err(|_| eyre!("invalid hex in genesis validators root: {hex}"))?;
    }

    let mut output = String::with_capacity(66);
    output.push_str("0x");
    for byte in &bytes {
        use std::fmt::Write;
        write!(&mut output, "{:02x}", byte).expect("write to string");
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_fork_version() {
        let result = parse_fork_version_hex("0x01020304").unwrap();
        assert_eq!(result, [0x01, 0x02, 0x03, 0x04]);
    }

    #[test]
    fn rejects_invalid_fork_version_length() {
        let err = parse_fork_version_hex("0x01").unwrap_err();
        assert!(err.to_string().contains("8 hex characters"));
    }

    #[test]
    fn parses_genesis_root_and_canonicalizes_prefix() {
        let input = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
        let output = parse_genesis_validators_root_hex(input).unwrap();
        assert!(output.starts_with("0x"));
        assert_eq!(output.len(), 66);
    }
}
