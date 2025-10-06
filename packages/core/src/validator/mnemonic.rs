use bip39::rand::rngs::OsRng;
use bip39::{Language, Mnemonic};
use eyre::Result;
use zeroize::Zeroizing;

pub struct GeneratedMnemonic {
    phrase: String,
    seed: Zeroizing<[u8; 64]>,
}

impl GeneratedMnemonic {
    pub fn generate() -> Result<Self> {
        let mnemonic = Mnemonic::generate_in_with(&mut OsRng, Language::English, 24)?;
        let phrase = mnemonic.to_string();
        let seed = mnemonic.to_seed("");
        let mut bytes = Zeroizing::new([0u8; 64]);
        bytes.copy_from_slice(&seed);
        Ok(Self {
            phrase,
            seed: bytes,
        })
    }

    pub fn phrase(&self) -> &str {
        &self.phrase
    }

    pub fn seed(&self) -> &[u8; 64] {
        &self.seed
    }
}
