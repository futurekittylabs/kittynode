mod input_validation;
mod lighthouse;

use std::io::{Write, stdout};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use self::lighthouse::{KeygenConfig, generate_validator_files};
use alloy_primitives::U256;
use alloy_primitives::utils::{Unit, format_units, keccak256};
use bip39::{Language, Mnemonic, MnemonicType, Seed as Bip39Seed};
use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use dialoguer::{Confirm, Input, Password, Select, theme::ColorfulTheme};
use eth2_network_config::HARDCODED_NET_NAMES;
use eyre::{Result, eyre};
use hmac::{Hmac, Mac};
use k256::{
    FieldBytes, Scalar, SecretKey,
    elliptic_curve::{PrimeField, sec1::ToEncodedPoint},
};
use sha2::Sha512;
use std::path::PathBuf;
use std::str::FromStr;
use tracing::{debug, error};
use types::Address;
use zeroize::Zeroizing;

use input_validation::{
    normalize_withdrawal_address, parse_deposit_amount_gwei, parse_validator_count,
    validate_password,
};

const CONNECTIVITY_PROBES: &[(&str, u16)] = &[
    ("one.one.one.one", 443),
    ("8.8.8.8", 53),
    ("www.google.com", 80),
];
const CONNECTIVITY_TIMEOUT: Duration = Duration::from_secs(2);

fn desired_supported_networks() -> Vec<&'static str> {
    const DESIRED: &[&str] = &["hoodi", "sepolia"];
    DESIRED
        .iter()
        .copied()
        .filter(|n| HARDCODED_NET_NAMES.contains(n))
        .collect()
}

pub async fn keygen() -> Result<()> {
    let theme = ColorfulTheme::default();

    // Pre-check warnings block
    let mut warnings: Vec<String> = Vec::new();

    // Internet connectivity warning
    if check_internet_connectivity() {
        warnings.push(
            "Internet connectivity detected. You should never generate keys on a device that's ever been connected to the internet.".to_string(),
        );
    }

    // Swap detection or limitation
    #[cfg(target_os = "linux")]
    {
        if swap_active() {
            warnings.push(
                "System swap detected. Sensitive key material can be written to disk via swap and persist.".to_string(),
            );
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        warnings.push(
            "Swap detection is unavailable on this platform. Ensure swap or pagefile is disabled before generating keys.".to_string(),
        );
    }

    // Non-Unix permission enforcement limitation
    #[cfg(not(unix))]
    {
        warnings.push(
            "This platform does not support enforcing POSIX file permissions for keystores. Ensure the output directory is protected.".to_string(),
        );
    }

    if !warnings.is_empty() {
        println!("WARNING:");
        for w in &warnings {
            println!(" - {}", w);
        }
        let proceed = Confirm::with_theme(&theme)
            .with_prompt("Proceed despite the above warnings?")
            .default(false)
            .interact()?;
        if !proceed {
            println!("Aborting validator key generation.");
            return Ok(());
        }
    }

    let validator_count_input = Input::<String>::with_theme(&theme)
        .with_prompt("How many validators do you wish to run?")
        .default("1".to_string())
        .validate_with(|text: &String| {
            parse_validator_count(text)
                .map(|_| ())
                .map_err(|error| error.to_string())
        })
        .interact_text()?;
    let validator_count = parse_validator_count(&validator_count_input)?;

    let network_labels = desired_supported_networks();
    if network_labels.is_empty() {
        return Err(eyre!(
            "No supported networks are available in this Lighthouse build. Please upgrade Lighthouse (and this CLI if needed)"
        ));
    }
    let network_index = Select::with_theme(&theme)
        .with_prompt("Select the network")
        .default(0)
        .items(&network_labels)
        .interact()?;
    let network = network_labels
        .get(network_index)
        .copied()
        .ok_or_else(|| eyre!("Invalid network selection"))?;

    let add_withdrawal_address = Confirm::with_theme(&theme)
        .with_prompt("Add a withdrawal address (y/n)")
        .default(true)
        .interact()?;
    let (withdrawal_address_display, withdrawal_address_normalized) = if add_withdrawal_address {
        let input = Input::<String>::with_theme(&theme)
            .with_prompt("Enter the withdrawal address")
            .validate_with(|text: &String| {
                normalize_withdrawal_address(text)
                    .map(|_| ())
                    .map_err(|error| error.to_string())
            })
            .interact_text()?;
        let normalized = normalize_withdrawal_address(&input)?;
        (Some(input.trim().to_string()), Some(normalized))
    } else {
        (None, None)
    };

    let compounding = Confirm::with_theme(&theme)
        .with_prompt("Use 0x02 compounding validators?")
        .default(true)
        .interact()?;

    // Deposit per validator (ETH). Only prompt when using compounding validators to match deposit-cli UX.
    let deposit_amount_gwei_per_validator: u64 = if compounding {
        let input = Input::<String>::with_theme(&theme)
            .with_prompt("Deposit per validator (ETH)")
            .default("32".to_string())
            .validate_with(|text: &String| {
                const MIN_DEPOSIT_GWEI: u64 = 1_000_000_000; // 1 ETH
                const MAX_DEPOSIT_GWEI: u64 = 2_048_000_000_000; // 2048 ETH per deposit entry
                match parse_deposit_amount_gwei(text) {
                    Ok(gwei) => {
                        if gwei < MIN_DEPOSIT_GWEI {
                            Err("Per-validator deposit must be at least 1 ETH".to_string())
                        } else if gwei > MAX_DEPOSIT_GWEI {
                            Err("Per-validator deposit cannot exceed 2048 ETH".to_string())
                        } else {
                            Ok(())
                        }
                    }
                    Err(error) => Err(error.to_string()),
                }
            })
            .interact_text()?;
        parse_deposit_amount_gwei(&input)?
    } else {
        32_000_000_000 // exactly 32 ETH for non-compounding
    };
    let validator_count_u64 = u64::from(validator_count);
    let total_deposit_gwei = deposit_amount_gwei_per_validator * validator_count_u64;
    let deposit_amount_per_validator_eth_str =
        format_eth_trimmed_from_gwei(deposit_amount_gwei_per_validator);

    // Allow user to select output directory for keys.
    let output_dir_input = Input::<String>::with_theme(&theme)
        .with_prompt("Output directory for validator keys")
        .default("./validator-keys".to_string())
        .interact_text()?;
    let output_dir = PathBuf::from(output_dir_input.trim());

    println!("Validator key generation summary:");
    println!("  Validators: {validator_count}");
    println!("  Network: {}", network);
    let withdrawal_summary = withdrawal_address_display
        .as_deref()
        .unwrap_or("First generated Ethereum address");
    println!("  Withdrawal address: {}", withdrawal_summary);
    println!(
        "  0x02 compounding validators: {}",
        if compounding { "yes" } else { "no" }
    );
    let total_deposit_eth_str = format_eth_trimmed_from_gwei(total_deposit_gwei);
    println!("  Total deposit: {} ETH", total_deposit_eth_str);
    println!(
        "  Deposit per validator: {} ETH",
        deposit_amount_per_validator_eth_str
    );
    println!("  Output directory: {}", output_dir.display());

    let confirm_details = Confirm::with_theme(&theme)
        .with_prompt("Are these details correct?")
        .default(true)
        .interact()?;
    if !confirm_details {
        println!("Aborting validator key generation.");
        return Ok(());
    }

    let mnemonic = Mnemonic::new(MnemonicType::Words24, Language::English);
    let mnemonic_phrase = Zeroizing::new(mnemonic.to_string());
    drop(mnemonic);

    display_mnemonic_securely(mnemonic_phrase.as_str())?;
    let mnemonic_verified = validate_mnemonic_once(&theme, mnemonic_phrase.as_str())?;
    if let Err(error) = clear_clipboard() {
        error!("Failed to clear system clipboard, mnemonic may remain in clipboard: {error}");
    }
    if !mnemonic_verified {
        println!("✘ Mnemonic verification failed. Aborting validator key generation.");
        return Ok(());
    }
    println!("Mnemonic successfully verified!");

    let password = Password::with_theme(&theme)
        .with_prompt("Enter a password to secure the keystore")
        .with_confirmation("Re-enter the password to confirm", "Passwords do not match")
        .validate_with(|value: &String| validate_password(value).map_err(|error| error.to_string()))
        .interact()?;

    let password = Zeroizing::new(password);

    let withdrawal_address = resolve_withdrawal_address(
        withdrawal_address_normalized.as_deref(),
        mnemonic_phrase.as_str(),
    )?;
    if withdrawal_address_normalized.is_none() {
        println!(
            "Using withdrawal address derived from mnemonic: {:#x}",
            withdrawal_address
        );
    }
    let outcome = generate_validator_files(KeygenConfig {
        mnemonic_phrase,
        validator_count,
        withdrawal_address,
        network: network.to_string(),
        deposit_gwei: deposit_amount_gwei_per_validator,
        compounding,
        password,
        output_dir,
    })?;

    println!(
        "✔ Generated {} validator keystore(s):",
        outcome.keystore_paths.len()
    );
    for path in &outcome.keystore_paths {
        println!("   {}", path.display());
    }
    println!(
        "✔ Deposit data written to {}",
        outcome.deposit_data_path.display()
    );

    println!("Store the password safely—it is not saved anywhere else.");

    Ok(())
}

fn format_eth_trimmed_from_gwei(gwei: u64) -> String {
    // Defer conversion to alloy; trim only for display.
    let wei = U256::from(gwei) * Unit::GWEI.wei();
    match format_units(wei, "ether") {
        Ok(s) => {
            if s.contains('.') {
                let s = s.trim_end_matches('0').trim_end_matches('.');
                s.to_string()
            } else {
                s
            }
        }
        Err(_) => gwei.to_string(),
    }
}

fn check_internet_connectivity() -> bool {
    CONNECTIVITY_PROBES
        .iter()
        .any(|(host, port)| match (*host, *port).to_socket_addrs() {
            Ok(mut addrs) => {
                addrs.any(|addr| TcpStream::connect_timeout(&addr, CONNECTIVITY_TIMEOUT).is_ok())
            }
            Err(error) => {
                debug!("DNS resolution failed for {host}:{port}: {error}");
                false
            }
        })
}

#[cfg(target_os = "linux")]
fn swap_active() -> bool {
    if let Ok(s) = std::fs::read_to_string("/proc/swaps") {
        let mut lines = s.lines();
        let _ = lines.next(); // header
        return lines.any(|l| !l.trim().is_empty());
    }
    false
}

fn clear_clipboard() -> Result<()> {
    let mut clipboard = arboard::Clipboard::new()
        .map_err(|error| eyre!("Failed to open system clipboard: {error}"))?;
    clipboard
        .set_text(String::new())
        .map_err(|error| eyre!("Failed to clear clipboard contents: {error}"))?;
    Ok(())
}

fn resolve_withdrawal_address(user: Option<&str>, mnemonic: &str) -> Result<Address> {
    match user {
        Some(value) => Address::from_str(value)
            .map_err(|error| eyre!("Failed to parse withdrawal address: {error}")),
        None => default_withdrawal_address(mnemonic),
    }
}

fn default_withdrawal_address(mnemonic: &str) -> Result<Address> {
    let mnemonic = Mnemonic::from_phrase(mnemonic, Language::English)
        .map_err(|error| eyre!("Mnemonic phrase is invalid: {error}"))?;
    let seed = Bip39Seed::new(&mnemonic, "");
    derive_execution_address(seed.as_bytes())
        .map_err(|error| eyre!("Failed to derive withdrawal address from mnemonic: {error}"))
}

fn derive_execution_address(seed: &[u8]) -> Result<Address> {
    type HmacSha512 = Hmac<Sha512>;
    const HARDENED_OFFSET: u32 = 1 << 31;
    const DERIVATION_PATH: [u32; 5] = [
        44 | HARDENED_OFFSET,
        60 | HARDENED_OFFSET,
        HARDENED_OFFSET,
        0,
        0,
    ];

    let mut master_hmac = HmacSha512::new_from_slice(b"Bitcoin seed")
        .map_err(|error| eyre!("Failed to initialize master key derivation: {error}"))?;
    master_hmac.update(seed);
    let output = master_hmac.finalize().into_bytes();

    let mut secret_bytes = FieldBytes::default();
    secret_bytes.copy_from_slice(&output[..32]);
    let mut chain_code = [0u8; 32];
    chain_code.copy_from_slice(&output[32..]);
    let mut secret_key = SecretKey::from_bytes(&secret_bytes)
        .map_err(|error| eyre!("Invalid master private key: {error}"))?;

    for index in DERIVATION_PATH {
        let hardened = (index & HARDENED_OFFSET) != 0;
        let mut data = Vec::with_capacity(1 + 33 + 4);
        if hardened {
            data.push(0);
            data.extend_from_slice(secret_key.to_bytes().as_slice());
        } else {
            let compressed = secret_key.public_key().to_encoded_point(true);
            data.extend_from_slice(compressed.as_bytes());
        }
        data.extend_from_slice(&index.to_be_bytes());

        let mut hmac = HmacSha512::new_from_slice(&chain_code)
            .map_err(|error| eyre!("Failed to initialize child key derivation: {error}"))?;
        hmac.update(&data);
        let digest = hmac.finalize().into_bytes();

        let mut il = FieldBytes::default();
        il.copy_from_slice(&digest[..32]);
        let mut ir = [0u8; 32];
        ir.copy_from_slice(&digest[32..]);

        let tweak = Scalar::from_repr(il)
            .into_option()
            .ok_or_else(|| eyre!("Derived scalar is out of range"))?;
        if tweak.is_zero().into() {
            return Err(eyre!("Derived scalar is zero"));
        }

        let parent_scalar = Scalar::from_repr(secret_key.to_bytes())
            .into_option()
            .ok_or_else(|| eyre!("Parent scalar is out of range"))?;
        let child_scalar = tweak + parent_scalar;
        if child_scalar.is_zero().into() {
            return Err(eyre!("Child scalar is zero"));
        }

        let child_bytes = child_scalar.to_bytes();
        secret_key = SecretKey::from_bytes(&child_bytes)
            .map_err(|error| eyre!("Failed to construct child private key: {error}"))?;
        chain_code = ir;
    }

    let public_key = secret_key.public_key();
    let uncompressed = public_key.to_encoded_point(false);
    let hash = keccak256(&uncompressed.as_bytes()[1..]);
    Ok(Address::from_slice(&hash[12..]))
}

fn display_mnemonic_securely(mnemonic: &str) -> Result<()> {
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let result = (|| -> Result<()> {
        execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
        println!("IMPORTANT: Write down this mnemonic in a safe place. It will not be saved.\n");
        println!("Mnemonic phrase:\n");
        println!("{mnemonic}\n");
        println!("Press ENTER after you have written down the mnemonic to continue.");
        stdout.flush()?;

        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer)?;
        Ok(())
    })();
    execute!(stdout, LeaveAlternateScreen)?;
    stdout.flush()?;
    result
}

fn validate_mnemonic_once(theme: &ColorfulTheme, mnemonic: &str) -> Result<bool> {
    let attempt = capture_mnemonic_securely(theme)?;
    Ok(normalize_mnemonic(&attempt) == normalize_mnemonic(mnemonic))
}

fn capture_mnemonic_securely(theme: &ColorfulTheme) -> Result<String> {
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let result = (|| -> Result<String> {
        execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
        println!("Please re-enter your mnemonic to confirm.\n");
        stdout.flush()?;

        Input::<String>::with_theme(theme)
            .with_prompt("Mnemonic phrase")
            .interact_text()
            .map_err(eyre::Report::from)
    })();
    execute!(stdout, LeaveAlternateScreen)?;
    stdout.flush()?;
    result
}

fn normalize_mnemonic(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::{default_withdrawal_address, normalize_mnemonic, resolve_withdrawal_address};
    use eyre::Result;
    use std::str::FromStr;
    use types::Address;

    #[test]
    fn normalize_mnemonic_collapses_whitespace() {
        assert_eq!(
            normalize_mnemonic("word1  word2\n\tword3"),
            "word1 word2 word3"
        );
    }

    #[test]
    fn resolve_withdrawal_address_prefers_user_value() -> Result<()> {
        let expected = Address::from_str("0x48fe05daea0f8cc6958a72522db42b2edb3fda1a")?;
        let resolved = resolve_withdrawal_address(
            Some("0x48fe05daea0f8cc6958a72522db42b2edb3fda1a"),
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        )?;
        assert_eq!(resolved, expected);
        Ok(())
    }

    #[test]
    fn resolve_withdrawal_address_defaults_to_first_account() -> Result<()> {
        let expected = Address::from_str("0x9858effd232b4033e47d90003d41ec34ecaeda94")?;
        let resolved = resolve_withdrawal_address(
            None,
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        )?;
        assert_eq!(resolved, expected);
        Ok(())
    }

    #[test]
    fn default_withdrawal_address_derives_first_account() -> Result<()> {
        let expected = Address::from_str("0x9858effd232b4033e47d90003d41ec34ecaeda94")?;
        let derived = default_withdrawal_address(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        )?;
        assert_eq!(derived, expected);
        Ok(())
    }
}
