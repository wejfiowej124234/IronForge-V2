use anyhow::{anyhow, Result};
use bip39::{Language, Mnemonic};
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct MnemonicSecret {
    phrase: String,
}

impl MnemonicSecret {
    pub fn new(phrase: String) -> Self {
        Self { phrase }
    }

    pub fn as_str(&self) -> &str {
        &self.phrase
    }

    pub fn to_seed(&self, passphrase: &str) -> Vec<u8> {
        // Use parse_in to validate and get Mnemonic object
        if let Ok(mnemonic) = Mnemonic::parse_in(Language::English, &self.phrase) {
            mnemonic.to_seed(passphrase).to_vec()
        } else {
            vec![]
        }
    }
}

pub fn generate_mnemonic(word_count: usize) -> Result<MnemonicSecret> {
    // bip39 2.0 generate_in takes language and word count
    let mnemonic = Mnemonic::generate_in(Language::English, word_count)
        .map_err(|e| anyhow!("Failed to generate mnemonic: {}", e))?;
    Ok(MnemonicSecret::new(mnemonic.to_string()))
}

/// 验证助记词
/// 为未来扩展准备的验证函数
#[allow(dead_code)] // 为未来扩展准备
pub fn validate_mnemonic(phrase: &str) -> Result<()> {
    Mnemonic::parse_in(Language::English, phrase)
        .map(|_| ())
        .map_err(|e| anyhow!("Invalid mnemonic: {}", e))
}
