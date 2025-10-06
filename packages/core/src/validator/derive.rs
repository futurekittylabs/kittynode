use eyre::{Result, eyre};
use num_bigint::BigUint;

pub fn biguint_to_bytes32(value: &BigUint) -> Result<[u8; 32]> {
    let bytes = value.to_bytes_be();
    if bytes.len() > 32 {
        return Err(eyre!("derived key is longer than 32 bytes"));
    }
    let mut output = [0u8; 32];
    let start = 32 - bytes.len();
    output[start..].copy_from_slice(&bytes);
    Ok(output)
}
