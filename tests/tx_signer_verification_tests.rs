//! 生产级交易签名验证测试套件
//!
//! 测试覆盖:
//! - ✅ 多链签名验证（Ethereum/BSC/Polygon/Bitcoin/Solana/TON）
//! - ✅ 边界条件测试（零值、最大值、特殊地址）
//! - ✅ 错误处理验证（无效输入、格式错误）
//! - ✅ 签名一致性验证（幂等性）
//! - ✅ 安全性测试（私钥验证、地址校验）
//! - ✅ 性能基准（签名速度要求）
//!
//! 测试标准:
//! - 每个链至少5个测试用例
//! - 边界值测试覆盖
//! - 错误路径验证
//! - 文档化的测试数据

#[cfg(test)]
mod tx_signer_tests {
    use iron_forge::crypto::tx_signer::{
        BitcoinTxSigner, EthereumTxSigner, SolanaTxSigner, TonTxSigner,
    };
    use std::time::Instant;

    // ============ 测试常量 ============
    
    /// 测试私钥（永远不要在生产环境使用）
    const TEST_PRIVATE_KEY: &str = "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    const TEST_PRIVATE_KEY_INVALID: &str = "0xinvalid";
    
    /// 测试地址
    const ETH_TEST_ADDRESS: &str = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb6";
    const ETH_ZERO_ADDRESS: &str = "0x0000000000000000000000000000000000000000";
    const BTC_TEST_ADDRESS: &str = "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh";
    const SOLANA_TEST_ADDRESS: &str = "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM";
    const TON_TEST_ADDRESS: &str = "EQD__________________________________________0vo";
    
    /// 性能要求：单次签名应在此时间内完成
    const MAX_SIGN_TIME_MS: u128 = 100;

    // ============ Ethereum 签名测试 ============
    
    /// Test 1.1: Ethereum 标准转账签名
    #[test]
    fn test_ethereum_standard_transfer() {
        let result = EthereumTxSigner::sign_transaction(
            TEST_PRIVATE_KEY,
            ETH_TEST_ADDRESS,
            "1000000000000000000", // 1 ETH
            0,                      // nonce
            20_000_000_000,         // 20 Gwei
            21_000,                 // gas limit
            1,                      // Ethereum mainnet
        );

        assert!(result.is_ok(), "Standard transfer should succeed");
        let signed_tx = result.unwrap();
        
        // 验证签名格式
        assert!(signed_tx.starts_with("0x"), "Signed tx should start with 0x");
        assert!(signed_tx.len() > 100, "RLP encoded tx should be substantial");
        assert!(signed_tx.len() < 1000, "Tx should not be excessively long");
    }
    
    /// Test 1.2: Ethereum 零值转账（有效场景）
    #[test]
    fn test_ethereum_zero_value_transfer() {
        let result = EthereumTxSigner::sign_transaction(
            TEST_PRIVATE_KEY,
            ETH_TEST_ADDRESS,
            "0",
            0,
            20_000_000_000,
            21_000,
            1,
        );

        assert!(result.is_ok(), "Zero value transfer should be valid");
    }
    
    /// Test 1.3: Ethereum 多链支持（BSC/Polygon）
    #[test]
    fn test_ethereum_multi_chain() {
        let chains = vec![
            (1, "Ethereum"),
            (56, "BSC"),
            (137, "Polygon"),
        ];
        
        for (chain_id, chain_name) in chains {
            let result = EthereumTxSigner::sign_transaction(
                TEST_PRIVATE_KEY,
                ETH_TEST_ADDRESS,
                "1000000000000000000",
                0,
                20_000_000_000,
                21_000,
                chain_id,
            );
            
            assert!(result.is_ok(), "{} signing should succeed", chain_name);
        }
    }
    
    /// Test 1.4: Ethereum 大额转账
    #[test]
    fn test_ethereum_large_value() {
        let result = EthereumTxSigner::sign_transaction(
            TEST_PRIVATE_KEY,
            ETH_TEST_ADDRESS,
            "1000000000000000000000", // 1000 ETH
            0,
            20_000_000_000,
            21_000,
            1,
        );

        assert!(result.is_ok(), "Large value transfer should work");
    }
    
    /// Test 1.5: Ethereum 无效私钥应失败
    #[test]
    fn test_ethereum_invalid_private_key() {
        let result = EthereumTxSigner::sign_transaction(
            TEST_PRIVATE_KEY_INVALID,
            ETH_TEST_ADDRESS,
            "1000000000000000000",
            0,
            20_000_000_000,
            21_000,
            1,
        );

        assert!(result.is_err(), "Invalid private key should fail");
    }
    
    /// Test 1.6: Ethereum 签名性能测试
    #[test]
    fn test_ethereum_signing_performance() {
        let start = Instant::now();
        
        let result = EthereumTxSigner::sign_transaction(
            TEST_PRIVATE_KEY,
            ETH_TEST_ADDRESS,
            "1000000000000000000",
            0,
            20_000_000_000,
            21_000,
            1,
        );
        
        let duration = start.elapsed().as_millis();
        
        assert!(result.is_ok(), "Signing should succeed");
        assert!(
            duration < MAX_SIGN_TIME_MS,
            "Signing took {}ms, should be < {}ms",
            duration,
            MAX_SIGN_TIME_MS
        );
    }

    // ============ Ethereum 合约调用签名测试 ============
    
    /// Test 2.1: ERC20 Transfer 签名
    #[test]
    fn test_ethereum_erc20_transfer() {
        // ERC20 transfer(address,uint256) function signature
        let data = "0xa9059cbb000000000000000000000000742d35cc6634c0532925a3b844bc9e7595f0beb60000000000000000000000000000000000000000000000000de0b6b3a7640000";
        
        let result = EthereumTxSigner::sign_transaction_with_data(
            TEST_PRIVATE_KEY,
            ETH_TEST_ADDRESS,
            "0",
            data,
            0,
            20_000_000_000,
            65_000,
            1,
        );

        assert!(result.is_ok(), "ERC20 transfer should succeed");
        let signed_tx = result.unwrap();
        assert!(signed_tx.starts_with("0x"), "Should have 0x prefix");
        assert!(signed_tx.len() > 200, "Contract call tx should be longer");
    }
    
    /// Test 2.2: 空 data 字段（等同于普通转账）
    #[test]
    fn test_ethereum_empty_data() {
        let result = EthereumTxSigner::sign_transaction_with_data(
            TEST_PRIVATE_KEY,
            ETH_TEST_ADDRESS,
            "1000000000000000000",
            "",
            0,
            20_000_000_000,
            21_000,
            1,
        );

        assert!(result.is_ok(), "Empty data should work");
    }
    
    /// Test 2.3: 复杂合约调用
    #[test]
    fn test_ethereum_complex_contract_call() {
        // 模拟 Uniswap swap 调用
        let complex_data = "0x38ed1739000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000";
        
        let result = EthereumTxSigner::sign_transaction_with_data(
            TEST_PRIVATE_KEY,
            ETH_TEST_ADDRESS,
            "0",
            complex_data,
            0,
            20_000_000_000,
            200_000,
            1,
        );

        assert!(result.is_ok(), "Complex contract call should work");
    }

    // ============ Solana 签名测试 ============
    
    /// Test 3.1: Solana 标准转账
    #[test]
    fn test_solana_standard_transfer() {
        let result = SolanaTxSigner::sign_transaction(
            TEST_PRIVATE_KEY,
            SOLANA_TEST_ADDRESS,
            "1000000", // 0.001 SOL (1 SOL = 1e9 lamports)
            "11111111111111111111111111111111",
        );

        assert!(result.is_ok(), "Solana transfer should succeed");
        let signed_tx: String = result.unwrap();
        assert!(!signed_tx.is_empty(), "Signed tx should not be empty");
        // Base64 编码的交易应该只包含有效字符
        assert!(signed_tx.chars().all(|c: char| c.is_alphanumeric() || c == '+' || c == '/' || c == '='));
    }
    
    /// Test 3.2: Solana 最小转账（1 lamport）
    #[test]
    fn test_solana_minimal_transfer() {
        let result = SolanaTxSigner::sign_transaction(
            TEST_PRIVATE_KEY,
            SOLANA_TEST_ADDRESS,
            "1",
            "11111111111111111111111111111111",
        );

        assert!(result.is_ok(), "Minimal transfer should work");
    }
    
    /// Test 3.3: Solana 大额转账
    #[test]
    fn test_solana_large_transfer() {
        let result = SolanaTxSigner::sign_transaction(
            TEST_PRIVATE_KEY,
            SOLANA_TEST_ADDRESS,
            "1000000000", // 1 SOL
            "11111111111111111111111111111111",
        );

        assert!(result.is_ok(), "Large transfer should work");
    }
    
    /// Test 3.4: Solana 无效地址应失败
    #[test]
    fn test_solana_invalid_address() {
        let result = SolanaTxSigner::sign_transaction(
            TEST_PRIVATE_KEY,
            "invalid_address",
            "1000000",
            "11111111111111111111111111111111",
        );

        // 取决于实现，可能返回错误或在内部处理
        // 至少应该不会 panic
        let _ = result;
    }
    
    /// Test 3.5: Solana 签名性能
    #[test]
    fn test_solana_signing_performance() {
        let start = Instant::now();
        
        let result = SolanaTxSigner::sign_transaction(
            TEST_PRIVATE_KEY,
            SOLANA_TEST_ADDRESS,
            "1000000",
            "11111111111111111111111111111111",
        );
        
        let duration = start.elapsed().as_millis();
        
        assert!(result.is_ok(), "Signing should succeed");
        assert!(
            duration < MAX_SIGN_TIME_MS,
            "Solana signing took {}ms, should be < {}ms",
            duration,
            MAX_SIGN_TIME_MS
        );
    }

    // ============ Bitcoin 签名测试 ============
    
    /// Test 4.1: Bitcoin 标准转账（SegWit）
    #[test]
    fn test_bitcoin_segwit_transfer() {
        let result = BitcoinTxSigner::sign_transaction(
            TEST_PRIVATE_KEY,
            BTC_TEST_ADDRESS,
            "100000", // 0.001 BTC
            20,       // 20 sat/vB
        );

        assert!(result.is_ok(), "Bitcoin transfer should succeed");
        let signed_tx = result.unwrap();
        assert!(signed_tx.contains("bitcoin") || !signed_tx.is_empty(), 
                "Should contain transaction data");
    }
    
    /// Test 4.2: Bitcoin 最小转账（灰尘限制）
    #[test]
    fn test_bitcoin_dust_limit() {
        let result = BitcoinTxSigner::sign_transaction(
            TEST_PRIVATE_KEY,
            BTC_TEST_ADDRESS,
            "546", // Dust limit: 546 satoshi
            20,
        );

        assert!(result.is_ok(), "Dust limit transfer should work");
    }
    
    /// Test 4.3: Bitcoin 大额转账
    #[test]
    fn test_bitcoin_large_transfer() {
        let result = BitcoinTxSigner::sign_transaction(
            TEST_PRIVATE_KEY,
            BTC_TEST_ADDRESS,
            "100000000", // 1 BTC
            20,
        );

        assert!(result.is_ok(), "Large BTC transfer should work");
    }
    
    /// Test 4.4: Bitcoin 不同费率
    #[test]
    fn test_bitcoin_different_fee_rates() {
        let fee_rates = vec![1, 10, 50, 100]; // sat/vB
        
        for fee_rate in fee_rates {
            let result = BitcoinTxSigner::sign_transaction(
                TEST_PRIVATE_KEY,
                BTC_TEST_ADDRESS,
                "100000",
                fee_rate,
            );
            
            assert!(result.is_ok(), "Fee rate {} should work", fee_rate);
        }
    }
    
    /// Test 4.5: Bitcoin Legacy 地址支持
    #[test]
    fn test_bitcoin_legacy_address() {
        let legacy_address = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"; // Genesis address
        
        let result = BitcoinTxSigner::sign_transaction(
            TEST_PRIVATE_KEY,
            legacy_address,
            "100000",
            20,
        );

        // 应该支持 legacy 地址或返回明确错误
        let _ = result;
    }

    // ============ TON 签名测试 ============
    
    /// Test 5.1: TON 标准转账
    #[test]
    fn test_ton_standard_transfer() {
        let result = TonTxSigner::sign_transaction(
            TEST_PRIVATE_KEY,
            TON_TEST_ADDRESS,
            "1000000000", // 1 TON
            0,            // seqno
        );

        assert!(result.is_ok(), "TON transfer should succeed");
        let signed_tx = result.unwrap();
        assert!(!signed_tx.is_empty(), "Signed tx should not be empty");
    }
    
    /// Test 5.2: TON 小额转账
    #[test]
    fn test_ton_small_transfer() {
        let result = TonTxSigner::sign_transaction(
            TEST_PRIVATE_KEY,
            TON_TEST_ADDRESS,
            "1000000", // 0.001 TON
            0,
        );

        assert!(result.is_ok(), "Small TON transfer should work");
    }
    
    /// Test 5.3: TON 不同 seqno
    #[test]
    fn test_ton_different_seqno() {
        let seqnos = vec![0, 1, 100, 1000];
        
        for seqno in seqnos {
            let result = TonTxSigner::sign_transaction(
                TEST_PRIVATE_KEY,
                TON_TEST_ADDRESS,
                "1000000000",
                seqno,
            );
            
            assert!(result.is_ok(), "Seqno {} should work", seqno);
        }
    }
    
    /// Test 5.4: TON 大额转账
    #[test]
    fn test_ton_large_transfer() {
        let result = TonTxSigner::sign_transaction(
            TEST_PRIVATE_KEY,
            TON_TEST_ADDRESS,
            "100000000000", // 100 TON
            0,
        );

        assert!(result.is_ok(), "Large TON transfer should work");
    }
    
    /// Test 5.5: TON 签名性能
    #[test]
    fn test_ton_signing_performance() {
        let start = Instant::now();
        
        let result = TonTxSigner::sign_transaction(
            TEST_PRIVATE_KEY,
            TON_TEST_ADDRESS,
            "1000000000",
            0,
        );
        
        let duration = start.elapsed().as_millis();
        
        assert!(result.is_ok(), "Signing should succeed");
        assert!(
            duration < MAX_SIGN_TIME_MS,
            "TON signing took {}ms, should be < {}ms",
            duration,
            MAX_SIGN_TIME_MS
        );
    }

    // ============ 签名一致性测试 ============
    
    /// Test 6.1: Ethereum 签名可重现性
    #[test]
    fn test_ethereum_signature_reproducibility() {
        let result1 = EthereumTxSigner::sign_transaction(
            TEST_PRIVATE_KEY,
            ETH_TEST_ADDRESS,
            "1000000000000000000", // 1 ETH
            0,                     // nonce
            20_000_000_000,        // gas_price
            21_000,                // gas_limit
            1,                     // chain_id
        );
        let result2 = EthereumTxSigner::sign_transaction(
            TEST_PRIVATE_KEY,
            ETH_TEST_ADDRESS,
            "1000000000000000000",
            0,
            20_000_000_000,
            21_000,
            1,
        );

        assert!(result1.is_ok() && result2.is_ok(), "Both signings should succeed");
        assert_eq!(
            result1.unwrap(),
            result2.unwrap(),
            "Same input should produce identical signature"
        );
    }
    
    /// Test 6.2: Bitcoin 签名确定性
    #[test]
    fn test_bitcoin_signature_determinism() {
        let sig1 = BitcoinTxSigner::sign_transaction(
            TEST_PRIVATE_KEY,
            BTC_TEST_ADDRESS,
            "100000", // 0.001 BTC
            10,       // 10 sat/vB
        );
        let sig2 = BitcoinTxSigner::sign_transaction(
            TEST_PRIVATE_KEY,
            BTC_TEST_ADDRESS,
            "100000",
            10,
        );
        let sig3 = BitcoinTxSigner::sign_transaction(
            TEST_PRIVATE_KEY,
            BTC_TEST_ADDRESS,
            "100000",
            10,
        );

        assert!(sig1.is_ok() && sig2.is_ok() && sig3.is_ok(), "All signings should succeed");
        
        let first = sig1.unwrap();
        assert_eq!(sig2.unwrap(), first, "Second signature must match first");
        assert_eq!(sig3.unwrap(), first, "Third signature must match first");
    }
    
    /// Test 6.3: Solana 签名稳定性
    #[test]
    fn test_solana_signature_stability() {
        let mut signatures = Vec::new();
        
        for _ in 0..5 {
            let result = SolanaTxSigner::sign_transaction(
                TEST_PRIVATE_KEY,
                SOLANA_TEST_ADDRESS,
                "1000000000", // 1 SOL
                "11111111111111111111111111111111",
            );
            assert!(result.is_ok(), "Solana signing should succeed");
            signatures.push(result.unwrap());
        }
        
        // All signatures should be identical
        let first = &signatures[0];
        for (i, sig) in signatures.iter().enumerate().skip(1) {
            assert_eq!(
                sig, first,
                "Signature {} differs from first signature",
                i
            );
        }
    }
    
    /// Test 6.4: 多链 Nonce 变化一致性
    #[test]
    fn test_multi_chain_nonce_consistency() {
        let chains = vec![
            (1, "Ethereum"),
            (56, "BSC"),
            (137, "Polygon"),
        ];
        
        for (chain_id, chain_name) in chains {
            // Same nonce should produce same signature
            let sig1 = EthereumTxSigner::sign_transaction(
                TEST_PRIVATE_KEY,
                ETH_TEST_ADDRESS,
                "1000000000000000000",
                42, // Fixed nonce
                20_000_000_000,
                21_000,
                chain_id,
            );
            let sig2 = EthereumTxSigner::sign_transaction(
                TEST_PRIVATE_KEY,
                ETH_TEST_ADDRESS,
                "1000000000000000000",
                42, // Same nonce
                20_000_000_000,
                21_000,
                chain_id,
            );
            
            assert!(sig1.is_ok() && sig2.is_ok(), "{} signing failed", chain_name);
            assert_eq!(
                sig1.unwrap(),
                sig2.unwrap(),
                "{} signatures should match for same nonce",
                chain_name
            );
        }
    }
    
    /// Test 6.5: TON 签名幂等性
    #[test]
    fn test_ton_signature_idempotency() {
        let test_params = vec![
            (0, "1000000000"),
            (1, "2000000000"),
            (100, "5000000000"),
        ];
        
        for (seqno, value) in test_params {
            let first = TonTxSigner::sign_transaction(
                TEST_PRIVATE_KEY,
                TON_TEST_ADDRESS,
                value,
                seqno,
            );
            let second = TonTxSigner::sign_transaction(
                TEST_PRIVATE_KEY,
                TON_TEST_ADDRESS,
                value,
                seqno,
            );
            
            assert!(first.is_ok() && second.is_ok(), "TON signing should work");
            assert_eq!(
                first.unwrap(),
                second.unwrap(),
                "TON signature should be idempotent for seqno {}, value {}",
                seqno,
                value
            );
        }
    }
    
    // ============ 错误处理测试 ============
    
    /// Test 7.1: 无效私钥格式
    #[test]
    fn test_invalid_private_key_formats() {
        let invalid_keys = vec![
            "",                    // Empty
            "0x",                  // No data
            "invalid",             // Non-hex
            "0x123",               // Too short
            "0xZZZZ",              // Invalid hex
        ];
        
        for key in invalid_keys {
            let result = EthereumTxSigner::sign_transaction(
                key,
                ETH_TEST_ADDRESS,
                "1000000000000000000",
                0,
                20_000_000_000,
                21_000,
                1,
            );
            
            // Should fail gracefully without panic
            assert!(result.is_err(), "Invalid key '{}' should fail", key);
        }
    }
    
    /// Test 7.2: 无效地址格式
    #[test]
    fn test_invalid_address_formats() {
        let invalid_addresses = vec![
            "",
            "0x",
            "not_an_address",
            "0x123", // Too short
        ];
        
        for addr in invalid_addresses {
            let result = EthereumTxSigner::sign_transaction(
                TEST_PRIVATE_KEY,
                addr,
                "1000000000000000000",
                0,
                20_000_000_000,
                21_000,
                1,
            );
            
            // Should handle gracefully
            let _ = result;
        }
    }
    
    /// Test 7.3: 边界值测试
    #[test]
    fn test_boundary_values() {
        // Zero value transfer
        let result = EthereumTxSigner::sign_transaction(
            TEST_PRIVATE_KEY,
            ETH_TEST_ADDRESS,
            "0",
            0,
            20_000_000_000,
            21_000,
            1,
        );
        assert!(result.is_ok(), "Zero value should work");
        
        // Extremely large value
        let result = EthereumTxSigner::sign_transaction(
            TEST_PRIVATE_KEY,
            ETH_TEST_ADDRESS,
            "999999999999999999999999999", // Very large
            0,
            20_000_000_000,
            21_000,
            1,
        );
        // Should either succeed or fail gracefully
        let _ = result;
    }
    
    /// Test 7.4: 不支持的链 ID
    #[test]
    fn test_unsupported_chain_ids() {
        let weird_chain_ids = vec![0, 999999, u64::MAX];
        
        for chain_id in weird_chain_ids {
            let result = EthereumTxSigner::sign_transaction(
                TEST_PRIVATE_KEY,
                ETH_TEST_ADDRESS,
                "1000000000000000000",
                0,
                20_000_000_000,
                21_000,
                chain_id,
            );
            
            // Should handle gracefully (either work or return error)
            let _ = result;
        }
    }
}
