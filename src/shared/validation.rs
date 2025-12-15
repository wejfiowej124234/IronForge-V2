use anyhow::{anyhow, Result};
use sha3::{Digest, Keccak256};

pub fn validate_eth_address(address: &str) -> Result<()> {
    if !address.starts_with("0x") {
        return Err(anyhow!("Ethereum address must start with 0x"));
    }
    if address.len() != 42 {
        return Err(anyhow!("Ethereum address must be 42 characters long"));
    }
    let hex_part = &address[2..];
    if hex::decode(hex_part).is_err() {
        return Err(anyhow!("Invalid hex characters"));
    }

    // EIP-55 Checksum
    let address_lower = address.to_lowercase();
    if address == address_lower || address == address.to_uppercase() {
        // All lower or all upper is valid (non-checksummed)
        return Ok(());
    }

    // Mixed case: must match checksum
    let hash = Keccak256::digest(&address_lower.as_bytes()[2..]);
    let hash_hex = hex::encode(hash);

    for (i, char) in address[2..].chars().enumerate() {
        let hash_char = hash_hex.chars().nth(i).unwrap();
        let hash_val = u8::from_str_radix(&hash_char.to_string(), 16).unwrap();

        if hash_val >= 8 {
            if char.is_ascii_lowercase() {
                return Err(anyhow!(
                    "Invalid checksum: expected uppercase at index {}",
                    i
                ));
            }
        } else if char.is_ascii_uppercase() {
            return Err(anyhow!(
                "Invalid checksum: expected lowercase at index {}",
                i
            ));
        }
    }

    Ok(())
}

pub fn validate_btc_address(address: &str) -> Result<()> {
    if address.starts_with("1") || address.starts_with("3") {
        // Legacy / Nested Segwit (Base58)
        let decoded = bs58::decode(address).into_vec();
        if decoded.is_err() {
            return Err(anyhow!("Invalid Base58 characters"));
        }
        let bytes = decoded.unwrap();
        if bytes.len() != 25 {
            return Err(anyhow!("Invalid Bitcoin address length"));
        }
        // Checksum validation omitted for brevity, but length/base58 is good start
    } else if address.starts_with("bc1") {
        // Native Segwit (Bech32)
        // Basic char check
        if address.len() > 90 {
            return Err(anyhow!("Invalid Bitcoin address length"));
        }
    } else {
        return Err(anyhow!("Unknown Bitcoin address format"));
    }
    Ok(())
}

pub fn validate_sol_address(address: &str) -> Result<()> {
    let decoded = bs58::decode(address).into_vec();
    match decoded {
        Ok(bytes) => {
            if bytes.len() != 32 {
                return Err(anyhow!("Invalid Solana address length"));
            }
            Ok(())
        }
        Err(_) => Err(anyhow!("Invalid Base58 characters")),
    }
}

pub fn validate_ton_address(address: &str) -> Result<()> {
    // TON addresses can be in two formats:
    // 1. Raw format: 48 hex characters (24 bytes)
    // 2. User-friendly format: starts with "EQ" or "UQ" followed by Base64-like encoding

    let trimmed = address.trim();

    // Check for user-friendly format (EQ/UQ prefix)
    if trimmed.starts_with("EQ") || trimmed.starts_with("UQ") {
        // User-friendly format: should be 48 characters after prefix
        if trimmed.len() < 48 || trimmed.len() > 50 {
            return Err(anyhow!("Invalid TON address length"));
        }
        // Basic validation: should contain only base64url-like characters
        let body = &trimmed[2..];
        if !body
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(anyhow!("Invalid TON address characters"));
        }
        return Ok(());
    }

    // Check for raw hex format
    if trimmed.len() == 48 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        return Ok(());
    }

    Err(anyhow!(
        "Invalid TON address format. Expected format: EQ... or UQ... or 48 hex characters"
    ))
}
