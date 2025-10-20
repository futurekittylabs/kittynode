use eyre::{Result, eyre};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EthereumConfig {
    pub network: Network,
    #[serde(default)]
    pub validator: Validator,
}

impl EthereumConfig {
    pub fn validate(&self) -> Result<()> {
        if self.validator.enabled {
            match self.validator.fee_recipient.as_ref() {
                Some(fee_recipient) if !fee_recipient.trim().is_empty() => {}
                _ => {
                    return Err(eyre!(
                        "Validator fee recipient is required when validator is enabled"
                    ));
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Network {
    Mainnet,
    Sepolia,
    Hoodi,
    Ephemery,
}

impl Network {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Network::Mainnet => "mainnet",
            Network::Sepolia => "sepolia",
            Network::Hoodi => "hoodi",
            Network::Ephemery => "ephemery",
        }
    }
}

impl fmt::Display for Network {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Network {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "mainnet" => Ok(Network::Mainnet),
            "sepolia" => Ok(Network::Sepolia),
            "hoodi" => Ok(Network::Hoodi),
            "ephemery" => Ok(Network::Ephemery),
            other => Err(eyre!("Unsupported Ethereum network: {other}")),
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Validator {
    pub enabled: bool,
    pub fee_recipient: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_and_validate_default_validator() {
        let raw = r#"
network = "mainnet"
"#;
        let cfg: EthereumConfig = toml::from_str(raw).expect("config should parse");
        assert_eq!(cfg.network, Network::Mainnet);
        assert!(!cfg.validator.enabled);
        assert!(cfg.validator.fee_recipient.is_none());
        cfg.validate().expect("validation should pass");
    }

    #[test]
    fn validator_requires_fee_recipient_when_enabled() {
        let raw = r#"
network = "sepolia"

[validator]
enabled = true
"#;
        let cfg: EthereumConfig = toml::from_str(raw).expect("config should parse");
        let err = cfg
            .validate()
            .expect_err("validation should fail without fee recipient");
        assert!(
            err.to_string()
                .contains("Validator fee recipient is required"),
            "unexpected error: {err:?}"
        );
    }

    #[test]
    fn unknown_fields_are_rejected() {
        let raw = r#"
network = "hoodi"
unexpected = "value"
"#;
        let parsed = toml::from_str::<EthereumConfig>(raw);
        assert!(parsed.is_err(), "unknown fields should be rejected");
    }
}
