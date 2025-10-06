mod input_validation;

use std::convert::TryInto;
use std::fs::{self, File};
use std::io::{Write, stdout};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use dialoguer::{Confirm, Input, Password, Select, theme::ColorfulTheme};
use eyre::{Result, WrapErr, eyre};
use hex::decode;
use kittynode_core::validator::{
    self, GeneratedMnemonic, KeygenRequest, Network as ValidatorNetwork,
};
use serde_json::to_writer_pretty;
use tracing::debug;
use zeroize::Zeroizing;

use input_validation::{
    normalize_withdrawal_address, parse_deposit_amount, parse_validator_count, validate_password,
};

const CONNECTIVITY_PROBES: &[(&str, u16)] = &[
    ("one.one.one.one", 443),
    ("8.8.8.8", 53),
    ("www.google.com", 80),
];
const CONNECTIVITY_TIMEOUT: Duration = Duration::from_secs(2);
const OUTPUT_DIR: &str = "validator_keys";

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

    let network_labels = ValidatorNetwork::labels();
    let network_index = Select::with_theme(&theme)
        .with_prompt("Select the network")
        .default(0)
        .items(network_labels)
        .interact()?;
    let network = ValidatorNetwork::all()
        .get(network_index)
        .copied()
        .ok_or_else(|| eyre!("Invalid network selection"))?;

    let compounding = Confirm::with_theme(&theme)
        .with_prompt("Use 0x02 compounding validators?")
        .default(true)
        .interact()?;

    let withdrawal_address = if compounding {
        None
    } else {
        let input = Input::<String>::with_theme(&theme)
            .with_prompt("Enter the withdrawal address")
            .validate_with(|text: &String| {
                normalize_withdrawal_address(text)
                    .map(|_| ())
                    .map_err(|error| error.to_string())
            })
            .interact_text()?;
        Some(normalize_withdrawal_address(&input)?)
    };

    let deposit_amount_input = Input::<String>::with_theme(&theme)
        .with_prompt("How much ETH do you want to deposit to these validators?")
        .default("32".to_string())
        .validate_with(|text: &String| {
            parse_deposit_amount(text)
                .map(|_| ())
                .map_err(|error| error.to_string())
        })
        .interact_text()?;
    let deposit_amount = parse_deposit_amount(&deposit_amount_input)?;
    let validator_count_u64 = u64::from(validator_count);
    let total_deposit_gwei = (deposit_amount * 1_000_000_000.0).round() as u64;
    if !total_deposit_gwei.is_multiple_of(validator_count_u64) {
        return Err(eyre!(
            "Deposit amount must be evenly divisible across validators when expressed in gwei"
        ));
    }
    let deposit_amount_gwei_per_validator = total_deposit_gwei / validator_count_u64;
    if deposit_amount_gwei_per_validator > 32_000_000_000 {
        return Err(eyre!("Per-validator deposit cannot exceed 32 ETH"));
    }
    let deposit_amount_per_validator_eth =
        deposit_amount_gwei_per_validator as f64 / 1_000_000_000.0;

    println!("Validator key generation summary:");
    println!("  Validators: {validator_count}");
    println!("  Network: {network}");
    println!(
        "  0x02 compounding validators: {}",
        if compounding { "yes" } else { "no" }
    );
    if let Some(address) = withdrawal_address.as_ref() {
        println!("  Withdrawal address: {address}");
    } else {
        println!("  Withdrawal credentials: compounding (0x02 prefix)");
    }
    println!("  Total deposit: {deposit_amount} ETH");
    println!(
        "  Deposit per validator: {:.9} ETH",
        deposit_amount_per_validator_eth
    );

    let confirm_details = Confirm::with_theme(&theme)
        .with_prompt("Are these details correct?")
        .default(true)
        .interact()?;
    if !confirm_details {
        println!("Aborting validator key generation.");
        return Ok(());
    }

    let mnemonic = GeneratedMnemonic::generate()?;
    display_mnemonic_securely(mnemonic.phrase())?;
    let mnemonic_verified = validate_mnemonic_once(&theme, mnemonic.phrase())?;
    clear_clipboard();
    if !mnemonic_verified {
        println!("✘ Mnemonic verification failed. Aborting validator key generation.");
        return Ok(());
    }
    println!("Mnemonic successfully verified!");

    let password = Zeroizing::new(
        Password::with_theme(&theme)
            .with_prompt("Enter a password to secure the keystore")
            .validate_with(|value: &String| {
                validate_password(value).map_err(|error| error.to_string())
            })
            .interact()?,
    );
    let password_for_confirmation = password.clone();
    Password::with_theme(&theme)
        .with_prompt("Re-enter the password to confirm")
        .validate_with(move |value: &String| {
            if value == password_for_confirmation.as_str() {
                Ok(())
            } else {
                Err("Passwords do not match".to_string())
            }
        })
        .interact()?;

    let withdrawal_bytes = withdrawal_address
        .as_deref()
        .map(withdrawal_address_to_bytes)
        .transpose()?;

    let output_dir = std::env::current_dir()?.join(OUTPUT_DIR);
    fs::create_dir_all(&output_dir).wrap_err("failed to create validator key output directory")?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .wrap_err("system time appears to be before UNIX_EPOCH")?
        .as_secs();

    let artifacts = validator::generate_validators(KeygenRequest {
        seed: mnemonic.seed(),
        password: password.as_str(),
        validator_indices: 0..u32::from(validator_count),
        network,
        deposit_amount_gwei: deposit_amount_gwei_per_validator,
        withdrawal_address: withdrawal_bytes,
        compounding,
    })?;

    drop(password);

    let mut deposits = Vec::with_capacity(artifacts.len());
    for artifact in &artifacts {
        let filename = format!(
            "keystore-m_12381_3600_{}_0_0-{timestamp}.json",
            artifact.index
        );
        let path = output_dir.join(&filename);
        let mut file = File::create(&path)
            .wrap_err_with(|| format!("failed to create keystore {filename}"))?;
        to_writer_pretty(&mut file, &artifact.keystore)
            .wrap_err("failed to write keystore JSON")?;
        file.write_all(
            b"
",
        )?;
        deposits.push(artifact.deposit.clone());
    }

    let deposit_path = output_dir.join(format!("deposit_data-{timestamp}.json"));
    let mut deposit_file =
        File::create(&deposit_path).wrap_err("failed to create deposit data file")?;
    to_writer_pretty(&mut deposit_file, &deposits).wrap_err("failed to write deposit data")?;
    deposit_file.write_all(
        b"
",
    )?;

    println!(
        "Generated {} keystore(s) in {}",
        artifacts.len(),
        output_dir.display()
    );
    println!("Deposit data saved to {}", deposit_path.display());

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
fn clear_clipboard() {}

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
        println!(
            "Please re-enter your mnemonic to confirm. The clipboard will be cleared afterwards.\n"
        );
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

fn withdrawal_address_to_bytes(address: &str) -> Result<[u8; 20]> {
    let body = address.trim_start_matches("0x");
    let bytes = decode(body).map_err(|_| eyre!("Failed to decode withdrawal address"))?;
    let array: [u8; 20] = bytes
        .try_into()
        .map_err(|_| eyre!("Withdrawal address must be exactly 20 bytes"))?;
    Ok(array)
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
