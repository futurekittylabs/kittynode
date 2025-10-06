use aes::Aes128;
use ctr::Ctr128BE;
use ctr::cipher::{KeyIvInit, StreamCipher};
use eyre::{Result, eyre};
use rand::RngCore;
use rand::rngs::OsRng;
use scrypt::Params;
use scrypt::scrypt;
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use unicode_normalization::UnicodeNormalization;
use uuid::Uuid;
use zeroize::Zeroizing;

pub struct KeystoreRequest<'a> {
    pub secret: Zeroizing<[u8; 32]>,
    pub password: &'a str,
    pub path: String,
    pub public_key_hex: String,
}

#[derive(Serialize)]
pub struct KeystoreResult {
    pub crypto: Crypto,
    pub description: String,
    pub pubkey: String,
    pub path: Option<String>,
    pub uuid: Uuid,
    pub version: u8,
}

#[derive(Serialize)]
pub struct Crypto {
    pub kdf: KdfModule,
    pub checksum: ChecksumModule,
    pub cipher: CipherModule,
}

#[derive(Serialize)]
pub struct KdfModule {
    pub function: &'static str,
    pub params: ScryptParamsJson,
    pub message: String,
}

#[derive(Serialize)]
pub struct ScryptParamsJson {
    pub dklen: u32,
    pub n: u32,
    pub r: u32,
    pub p: u32,
    pub salt: String,
}

#[derive(Serialize)]
pub struct ChecksumModule {
    pub function: &'static str,
    pub params: serde_json::Value,
    pub message: String,
}

#[derive(Serialize)]
pub struct CipherModule {
    pub function: &'static str,
    pub params: CipherParams,
    pub message: String,
}

#[derive(Serialize)]
pub struct CipherParams {
    pub iv: String,
}

type Aes128Ctr = Ctr128BE<Aes128>;

const SCRYPT_LOG_N: u8 = 18; // 2^18 = 262144
const SCRYPT_R: u32 = 8;
const SCRYPT_P: u32 = 1;
const DK_LEN: usize = 32;

pub fn encrypt_keystore(request: KeystoreRequest<'_>) -> Result<KeystoreResult> {
    let KeystoreRequest {
        secret,
        password,
        path,
        public_key_hex,
    } = request;

    if password.is_empty() {
        return Err(eyre!("password cannot be empty"));
    }

    let mut derived_key = Zeroizing::new([0u8; DK_LEN]);
    let mut salt = [0u8; DK_LEN];
    OsRng.fill_bytes(&mut salt);

    let params = Params::new(SCRYPT_LOG_N, SCRYPT_R, SCRYPT_P, DK_LEN)
        .map_err(|error| eyre!("invalid scrypt parameters: {error}"))?;

    let normalized: String = password.nfkd().filter(|ch| !ch.is_control()).collect();

    scrypt(normalized.as_bytes(), &salt, &params, derived_key.as_mut())?;

    let mut ciphertext = secret[..].to_vec();
    let mut iv = [0u8; 16];
    OsRng.fill_bytes(&mut iv);
    let mut cipher = Aes128Ctr::new((&derived_key[..16]).into(), (&iv).into());
    cipher.apply_keystream(&mut ciphertext);

    let mut hasher = Sha256::new();
    hasher.update(&derived_key[16..]);
    hasher.update(&ciphertext);
    let checksum = hasher.finalize();

    Ok(KeystoreResult {
        crypto: Crypto {
            kdf: KdfModule {
                function: "scrypt",
                params: ScryptParamsJson {
                    dklen: DK_LEN as u32,
                    n: 1u32 << SCRYPT_LOG_N as u32,
                    r: SCRYPT_R,
                    p: SCRYPT_P,
                    salt: hex::encode(salt),
                },
                message: String::new(),
            },
            checksum: ChecksumModule {
                function: "sha256",
                params: Value::Object(Default::default()),
                message: hex::encode(checksum),
            },
            cipher: CipherModule {
                function: "aes-128-ctr",
                params: CipherParams {
                    iv: hex::encode(iv),
                },
                message: hex::encode(ciphertext),
            },
        },
        description: String::new(),
        pubkey: public_key_hex,
        path: Some(path),
        uuid: Uuid::new_v4(),
        version: 4,
    })
}
