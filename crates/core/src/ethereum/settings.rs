use crate::packages::PackageConfig;

#[derive(Clone)]
pub(crate) struct EthereumSettings {
    pub uses_local_execution: bool,
    pub uses_local_consensus: bool,
    pub validator: Option<ValidatorSettings>,
    pub consensus_endpoint: Option<String>,
}

impl EthereumSettings {
    pub fn runs_local_node(&self) -> bool {
        self.uses_local_execution || self.uses_local_consensus
    }
}

#[derive(Clone)]
pub(crate) struct ValidatorSettings {
    pub fee_recipient: String,
}

pub(crate) fn selected_network(config: &PackageConfig) -> Option<&str> {
    config.values.get("network").map(String::as_str)
}

pub(crate) fn ethereum_settings_from_config(config: &PackageConfig) -> EthereumSettings {
    let execution_endpoint = config
        .values
        .get("execution_endpoint")
        .filter(|value| !value.is_empty())
        .cloned();
    let consensus_endpoint = config
        .values
        .get("consensus_endpoint")
        .filter(|value| !value.is_empty())
        .cloned();

    let validator_enabled = config
        .values
        .get("validator_enabled")
        .map(|value| value == "true")
        .unwrap_or(false);
    let validator_fee_recipient = config
        .values
        .get("validator_fee_recipient")
        .filter(|value| !value.is_empty())
        .cloned();

    EthereumSettings {
        uses_local_execution: execution_endpoint.is_none(),
        uses_local_consensus: consensus_endpoint.is_none(),
        validator: if validator_enabled {
            validator_fee_recipient.map(|fee_recipient| ValidatorSettings { fee_recipient })
        } else {
            None
        },
        consensus_endpoint,
    }
}
