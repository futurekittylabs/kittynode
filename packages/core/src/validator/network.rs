use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Network {
    Hoodi,
    Sepolia,
    Ephemery,
}

const HOODI_FORK_VERSION: [u8; 4] = 0x1000_0910_u32.to_be_bytes();
const SEPOLIA_FORK_VERSION: [u8; 4] = 0x9000_0069_u32.to_be_bytes();
const EPHEMERY_FORK_VERSION: [u8; 4] = 0x1000_101b_u32.to_be_bytes();
const DEPOSIT_CLI_VERSION: &str = "1.2.2";

impl Network {
    pub const fn labels() -> [&'static str; 3] {
        ["hoodi", "sepolia", "ephemery"]
    }

    pub const fn all() -> [Network; 3] {
        [Network::Hoodi, Network::Sepolia, Network::Ephemery]
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Network::Hoodi => "hoodi",
            Network::Sepolia => "sepolia",
            Network::Ephemery => "ephemery",
        }
    }

    pub const fn fork_version(self) -> [u8; 4] {
        match self {
            Network::Hoodi => HOODI_FORK_VERSION,
            Network::Sepolia => SEPOLIA_FORK_VERSION,
            Network::Ephemery => EPHEMERY_FORK_VERSION,
        }
    }

    pub const fn deposit_cli_version(self) -> &'static str {
        match self {
            Network::Hoodi | Network::Sepolia | Network::Ephemery => DEPOSIT_CLI_VERSION,
        }
    }
}

impl fmt::Display for Network {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
