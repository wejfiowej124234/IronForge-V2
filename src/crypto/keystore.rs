//! Keystore - Ethereum Keystore文件解析和解密
//! 支持Ethereum Keystore V3格式（scrypt和pbkdf2 KDF）

use anyhow::{anyhow, Result};
use hex;
use hmac::Hmac;
use pbkdf2::pbkdf2;
use scrypt::{scrypt, Params as ScryptParams};
use sha2::{Digest, Sha256};
use sha3::Keccak256;

/// Keystore结构
#[derive(Debug, Clone)]
pub struct Keystore {
    #[allow(dead_code)] // 用于 Keystore 版本信息
    pub version: u64,
    #[allow(dead_code)] // 用于 Keystore ID
    pub id: String,
    #[allow(dead_code)] // 用于 Keystore 地址
    pub address: String,
    pub crypto: CryptoParams,
}

/// 加密参数
#[derive(Debug, Clone)]
pub struct CryptoParams {
    pub cipher: String,
    pub cipherparams: CipherParams,
    pub ciphertext: Vec<u8>,
    #[allow(dead_code)] // 用于 KDF 类型信息
    pub kdf: String,
    pub kdfparams: KdfParams,
    pub mac: String,
}

/// 加密参数（IV等）
#[derive(Debug, Clone)]
pub struct CipherParams {
    pub iv: Vec<u8>,
}

/// KDF参数
#[derive(Debug, Clone)]
pub enum KdfParams {
    Scrypt {
        dklen: u32,
        n: u32,
        p: u32,
        r: u32,
        salt: Vec<u8>,
    },
    Pbkdf2 {
        c: u32,
        dklen: u32,
        prf: String,
        salt: Vec<u8>,
    },
}

/// 解析Keystore JSON
pub fn parse_keystore(json_str: &str) -> Result<Keystore> {
    let json: serde_json::Value =
        serde_json::from_str(json_str).map_err(|e| anyhow!("Invalid JSON: {}", e))?;

    let version = json
        .get("version")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| anyhow!("Missing or invalid 'version' field"))?;

    let id = json
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing 'id' field"))?
        .to_string();

    let address = json
        .get("address")
        .and_then(|v| v.as_str())
        .map(|s| s.trim_start_matches("0x").to_string())
        .unwrap_or_default();

    let crypto_obj = json
        .get("crypto")
        .ok_or_else(|| anyhow!("Missing 'crypto' field"))?;

    let cipher = crypto_obj
        .get("cipher")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing 'cipher' field"))?
        .to_string();

    let cipherparams_obj = crypto_obj
        .get("cipherparams")
        .ok_or_else(|| anyhow!("Missing 'cipherparams' field"))?;

    let iv_str = cipherparams_obj
        .get("iv")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing 'iv' in cipherparams"))?;
    let iv = hex::decode(iv_str.trim_start_matches("0x"))
        .map_err(|e| anyhow!("Invalid IV hex: {}", e))?;

    let ciphertext_str = crypto_obj
        .get("ciphertext")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing 'ciphertext' field"))?;
    let ciphertext = hex::decode(ciphertext_str.trim_start_matches("0x"))
        .map_err(|e| anyhow!("Invalid ciphertext hex: {}", e))?;

    let kdf = crypto_obj
        .get("kdf")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing 'kdf' field"))?
        .to_string();

    let kdfparams_obj = crypto_obj
        .get("kdfparams")
        .ok_or_else(|| anyhow!("Missing 'kdfparams' field"))?;

    let kdfparams = match kdf.as_str() {
        "scrypt" => {
            let dklen = kdfparams_obj
                .get("dklen")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| anyhow!("Missing 'dklen' in kdfparams"))?
                as u32;
            let n = kdfparams_obj
                .get("n")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| anyhow!("Missing 'n' in kdfparams"))? as u32;
            let p = kdfparams_obj
                .get("p")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| anyhow!("Missing 'p' in kdfparams"))? as u32;
            let r = kdfparams_obj
                .get("r")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| anyhow!("Missing 'r' in kdfparams"))? as u32;
            let salt_str = kdfparams_obj
                .get("salt")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing 'salt' in kdfparams"))?;
            let salt = hex::decode(salt_str.trim_start_matches("0x"))
                .map_err(|e| anyhow!("Invalid salt hex: {}", e))?;

            KdfParams::Scrypt {
                dklen,
                n,
                p,
                r,
                salt,
            }
        }
        "pbkdf2" => {
            let c = kdfparams_obj
                .get("c")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| anyhow!("Missing 'c' in kdfparams"))? as u32;
            let dklen = kdfparams_obj
                .get("dklen")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| anyhow!("Missing 'dklen' in kdfparams"))?
                as u32;
            let prf = kdfparams_obj
                .get("prf")
                .and_then(|v| v.as_str())
                .unwrap_or("hmac-sha256")
                .to_string();
            let salt_str = kdfparams_obj
                .get("salt")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing 'salt' in kdfparams"))?;
            let salt = hex::decode(salt_str.trim_start_matches("0x"))
                .map_err(|e| anyhow!("Invalid salt hex: {}", e))?;

            KdfParams::Pbkdf2 {
                c,
                dklen,
                prf,
                salt,
            }
        }
        _ => return Err(anyhow!("Unsupported KDF: {}", kdf)),
    };

    let mac = crypto_obj
        .get("mac")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing 'mac' field"))?
        .to_string();

    Ok(Keystore {
        version,
        id,
        address,
        crypto: CryptoParams {
            cipher,
            cipherparams: CipherParams { iv },
            ciphertext,
            kdf,
            kdfparams,
            mac,
        },
    })
}

/// 派生密钥（使用KDF）
fn derive_key(password: &str, kdfparams: &KdfParams) -> Result<Vec<u8>> {
    match kdfparams {
        KdfParams::Scrypt {
            dklen,
            n,
            p,
            r,
            salt,
        } => {
            let params = ScryptParams::new((*n).try_into().unwrap_or(15), *r, *p, *dklen as usize)
                .map_err(|e| anyhow!("Invalid scrypt parameters: {}", e))?;
            let mut derived_key = vec![0u8; *dklen as usize];
            scrypt(password.as_bytes(), salt, &params, &mut derived_key)
                .map_err(|e| anyhow!("Scrypt key derivation failed: {}", e))?;
            Ok(derived_key)
        }
        KdfParams::Pbkdf2 {
            c,
            dklen,
            prf,
            salt,
        } => {
            if prf != "hmac-sha256" {
                return Err(anyhow!("Unsupported PRF: {}", prf));
            }
            let mut derived_key = vec![0u8; *dklen as usize];
            let _ = pbkdf2::<Hmac<Sha256>>(password.as_bytes(), salt, *c, &mut derived_key);
            Ok(derived_key)
        }
    }
}

/// 验证MAC
fn verify_mac(derived_key: &[u8], ciphertext: &[u8], expected_mac: &str) -> Result<()> {
    // MAC = Keccak256(derived_key[16..32] + ciphertext)
    let mac_input = [&derived_key[16..32], ciphertext].concat();
    let mut hasher = Keccak256::new();
    hasher.update(&mac_input);
    let computed_mac = hasher.finalize();
    let computed_mac_hex = hex::encode(&computed_mac[..16]);

    let expected_mac_clean = expected_mac.trim_start_matches("0x");
    if computed_mac_hex != expected_mac_clean[..32.min(expected_mac_clean.len())] {
        return Err(anyhow!("MAC verification failed"));
    }

    Ok(())
}

/// 解密私钥
pub fn decrypt_keystore(keystore_json: &str, password: &str) -> Result<String> {
    let keystore = parse_keystore(keystore_json)?;

    // 1. 派生密钥
    let derived_key = derive_key(password, &keystore.crypto.kdfparams)?;

    // 2. 验证MAC
    verify_mac(
        &derived_key,
        &keystore.crypto.ciphertext,
        &keystore.crypto.mac,
    )?;

    // 3. 解密私钥
    let private_key = match keystore.crypto.cipher.as_str() {
        "aes-128-ctr" => {
            // AES-128-CTR解密
            use aes::Aes128;
            use ctr::cipher::{KeyIvInit, StreamCipher};

            let key = &derived_key[..16];
            let key_array: [u8; 16] = key.try_into().map_err(|_| anyhow!("Invalid key length"))?;
            let iv_array: [u8; 16] = keystore
                .crypto
                .cipherparams
                .iv
                .as_slice()
                .try_into()
                .map_err(|_| anyhow!("Invalid IV length"))?;

            let mut cipher = ctr::Ctr128BE::<Aes128>::new(
                key_array.as_slice().into(),
                iv_array.as_slice().into(),
            );
            let mut plaintext = keystore.crypto.ciphertext.clone();
            cipher.apply_keystream(&mut plaintext);
            plaintext
        }
        "aes-128-cbc" => {
            // AES-128-CBC解密（简化实现，使用aes-gcm的CBC模式）
            // 注意：实际应该使用aes的CbcDecryptor，但为了简化，这里使用基础实现
            return Err(anyhow!(
                "AES-128-CBC decryption not fully implemented. Please use AES-128-CTR keystore."
            ));
        }
        _ => return Err(anyhow!("Unsupported cipher: {}", keystore.crypto.cipher)),
    };

    // 4. 返回私钥（十六进制）
    Ok(format!("0x{}", hex::encode(private_key)))
}
