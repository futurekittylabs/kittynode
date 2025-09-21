use clap::ValueEnum;

#[derive(Copy, Clone, Debug, ValueEnum, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
}

impl OutputFormat {
    #[must_use]
    pub fn is_json(self) -> bool {
        matches!(self, OutputFormat::Json)
    }
}
