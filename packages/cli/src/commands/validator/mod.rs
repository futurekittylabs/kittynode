mod input_validation;
mod lighthouse;

use std::io::{Write, stdout};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use self::lighthouse::{KeygenConfig, generate_validator_files};
use bip39::{Language, Mnemonic, MnemonicType};
use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use dialoguer::{Confirm, Input, Password, Select, theme::ColorfulTheme};
use eth2_network_config::HARDCODED_NET_NAMES;
use eyre::{Result, eyre};
use std::path::PathBuf;
use std::str::FromStr;
use tracing::debug;
use types::Address;
use zeroize::Zeroizing;

use input_validation::{
    normalize_withdrawal_address, parse_deposit_amount, parse_deposit_amount_gwei,
    parse_validator_count, validate_password,
};

const CONNECTIVITY_PROBES: &[(&str, u16)] = &[
    ("one.one.one.one", 443),
    ("8.8.8.8", 53),
    ("www.google.com", 80),
];
const CONNECTIVITY_TIMEOUT: Duration = Duration::from_secs(2);

fn desired_supported_networks() -> Vec<&'static str> {
    // Our keygen flow currently targets Ethereum-family networks that use the mainnet spec
    // (e.g., sepolia, holesky, hoodi). Keep the UI list constrained and filter by availability.
    const DESIRED: &[&str] = &["hoodi", "sepolia"];
    DESIRED
        .iter()
        .copied()
        .filter(|n| HARDCODED_NET_NAMES.contains(n))
        .collect()
}

#[allow(clippy::manual_is_multiple_of)]
pub async fn keygen() -> Result<()> {
    let theme = ColorfulTheme::default();

    let has_internet = check_internet_connectivity();
    if has_internet {
        println!(
            "Warning: Internet connectivity detected. You should never generate keys on a device that's ever been connected to the internet."
        );
        let proceed = Confirm::with_theme(&theme)
            .with_prompt("Proceed despite being connected to the internet?")
            .default(false)
            .interact()?;
        if !proceed {
            println!("Aborting validator key generation.");
            return Ok(());
        }
    } else {
        println!("No internet connectivity detected.");
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
            "No supported networks are available in this Lighthouse build. Please upgrade Lighthouse (and this CLI if needed)."
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

    let withdrawal_address_input = Input::<String>::with_theme(&theme)
        .with_prompt("Enter the withdrawal address")
        .validate_with(|text: &String| {
            normalize_withdrawal_address(text)
                .map(|_| ())
                .map_err(|error| error.to_string())
        })
        .interact_text()?;
    let withdrawal_address = normalize_withdrawal_address(&withdrawal_address_input)?;

    let compounding = Confirm::with_theme(&theme)
        .with_prompt("Use 0x02 compounding validators?")
        .default(true)
        .interact()?;

    let deposit_amount_input = Input::<String>::with_theme(&theme)
        .with_prompt("How much ETH do you want to deposit to these validators?")
        .default("32".to_string())
        .validate_with(|text: &String| {
            parse_deposit_amount(text)
                .map(|_| ())
                .map_err(|error| error.to_string())
        })
        .interact_text()?;
    let deposit_amount_total_eth = parse_deposit_amount(&deposit_amount_input)?;
    let validator_count_u64 = u64::from(validator_count);
    let total_deposit_gwei = parse_deposit_amount_gwei(&deposit_amount_input)?;
    if total_deposit_gwei % validator_count_u64 != 0 {
        let per_val_floor = total_deposit_gwei / validator_count_u64;
        let suggested_total_up = (per_val_floor + 1) * validator_count_u64;
        let suggested_eth = suggested_total_up as f64 / 1_000_000_000.0;
        return Err(eyre!(
            "Total deposit must be evenly divisible by {validator_count} validators. Try {:.9} ETH instead",
            suggested_eth
        ));
    }
    let deposit_amount_gwei_per_validator = total_deposit_gwei / validator_count_u64;
    // Enforce per-validator deposit rules
    const THIRTY_TWO_ETH_GWEI: u64 = 32_000_000_000; // 32 ETH
    const TWO_THOUSAND_FORTY_EIGHT_ETH_GWEI: u64 = 2_048_000_000_000; // 2048 ETH
    if compounding {
        if deposit_amount_gwei_per_validator < THIRTY_TWO_ETH_GWEI {
            return Err(eyre!(
                "Per-validator deposit must be at least 32 ETH for compounding validators"
            ));
        }
        if deposit_amount_gwei_per_validator > TWO_THOUSAND_FORTY_EIGHT_ETH_GWEI {
            return Err(eyre!(
                "Per-validator deposit cannot exceed 2048 ETH for compounding validators"
            ));
        }
    } else if deposit_amount_gwei_per_validator != THIRTY_TWO_ETH_GWEI {
        return Err(eyre!(
            "Per-validator deposit must be exactly 32 ETH for non-compounding validators"
        ));
    }
    let deposit_amount_per_validator_eth =
        deposit_amount_gwei_per_validator as f64 / 1_000_000_000.0;

    // Allow user to select output directory for keys.
    let output_dir_input = Input::<String>::with_theme(&theme)
        .with_prompt("Output directory for validator keys")
        .default("./validator-keys".to_string())
        .interact_text()?;
    let output_dir = PathBuf::from(&output_dir_input);

    println!("Validator key generation summary:");
    println!("  Validators: {validator_count}");
    println!("  Network: {}", network);
    println!("  Withdrawal address: {withdrawal_address}");
    println!(
        "  0x02 compounding validators: {}",
        if compounding { "yes" } else { "no" }
    );
    println!("  Total deposit: {deposit_amount_total_eth} ETH");
    println!(
        "  Deposit per validator: {:.9} ETH",
        deposit_amount_per_validator_eth
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
    clear_clipboard();
    if !mnemonic_verified {
        println!("✘ Mnemonic verification failed. Aborting validator key generation.");
        return Ok(());
    }
    println!("Mnemonic successfully verified!");

    let password = Password::with_theme(&theme)
        .with_prompt("Enter a password to secure the keystore")
        .validate_with(|value: &String| validate_password(value).map_err(|error| error.to_string()))
        .interact()?;
    let password_ref = &password;
    let _ = Password::with_theme(&theme)
        .with_prompt("Re-enter the password to confirm")
        .validate_with(|value: &String| {
            if value.as_str() == password_ref.as_str() {
                Ok(())
            } else {
                Err("Passwords do not match".to_string())
            }
        })
        .interact()?;

    let password = Zeroizing::new(password);

    let withdrawal_address = Address::from_str(&withdrawal_address)
        .map_err(|error| eyre!("Failed to parse withdrawal address: {error}"))?;
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

// TODO: Implement clipboard clearing
fn clear_clipboard() {
    // Best-effort clipboard clearing to avoid leaving sensitive data around.
    // On platforms where setting an empty string is unsupported, ignore errors.
    if let Ok(mut clipboard) = arboard::Clipboard::new() {
        let _ = clipboard.set_text(String::new());
    }
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
    use super::normalize_mnemonic;

    #[test]
    fn normalize_mnemonic_collapses_whitespace() {
        assert_eq!(
            normalize_mnemonic("word1  word2\n\tword3"),
            "word1 word2 word3"
        );
    }
}
