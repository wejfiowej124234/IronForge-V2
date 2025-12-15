// Sign In With Ethereum (SIWE) Authentication Service
// EIP-4361 compliant wallet authentication

use crate::crypto::key_manager::KeyManager;
use crate::shared::error::{ApiError, AppError};
use crate::shared::state::AppState;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

// ---------------- Email/Password Auth (Backend-provided) ----------------

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegisterReq {
    pub email: String,
    pub password: String,
    pub confirm_password: String,
    #[serde(default)]
    pub username: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegisterResp {
    pub access_token: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoginReq {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoginResp {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub user: UserInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // 用于 SIWE 认证流程
pub struct SiweMessage {
    pub domain: String,
    pub address: String,
    pub statement: String,
    pub uri: String,
    pub version: String,
    pub chain_id: u64,
    pub nonce: String,
    pub issued_at: String,
    pub expiration_time: Option<String>,
    pub not_before: Option<String>,
}

impl fmt::Display for SiweMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} wants you to sign in with your Ethereum account:\n{}\n\n{}\n\nURI: {}\nVersion: {}\nChain ID: {}\nNonce: {}\nIssued At: {}",
            self.domain,
            self.address,
            self.statement,
            self.uri,
            self.version,
            self.chain_id,
            self.nonce,
            self.issued_at
        )?;

        if let Some(exp) = &self.expiration_time {
            write!(f, "\nExpiration Time: {}", exp)?;
        }

        if let Some(nbf) = &self.not_before {
            write!(f, "\nNot Before: {}", nbf)?;
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // 用于 SIWE 认证流程
struct ChallengeResponse {
    nonce: String,
    message: String,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)] // 用于 SIWE 认证流程
struct VerifyRequest {
    message: String,
    signature: String,
}

// VerifyResponse 已移除，直接使用 VerifyData
// deserialize 方法已自动提取 data 字段

#[derive(Clone, Copy)]
pub struct AuthService {
    app_state: AppState,
}

impl AuthService {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    /// Register via email/password and return access token
    pub async fn register_email(
        &self,
        email: &str,
        password: &str,
        confirm_password: &str,
    ) -> Result<RegisterResp, AppError> {
        // Validate password confirmation on frontend
        if password != confirm_password {
            return Err(AppError::Api(ApiError::ResponseError(
                "Passwords do not match".into(),
            )));
        }

        let api = self.app_state.get_api_client();
        // Backend expects: email, password, optional phone
        // Note: confirm_password is validated on frontend, not sent to backend
        #[derive(Debug, Serialize)]
        struct RegisterBackendReq {
            email: String,
            password: String,
            phone: Option<String>,
        }
        let payload = RegisterBackendReq {
            email: email.to_string(),
            password: password.to_string(),
            phone: None, // Optional phone number
        };

        // Backend returns: RegisterResp { access_token, user }
        let resp: RegisterResp = api
            .post("/api/v1/auth/register", &payload)
            .await
            .map_err(AppError::from)?;
        Ok(resp)
    }

    /// Login via email/password and return access token
    pub async fn login_email(&self, email: &str, password: &str) -> Result<LoginResp, AppError> {
        let api = self.app_state.get_api_client();
        let payload = LoginReq {
            email: email.to_string(),
            password: password.to_string(),
        };
        let resp: LoginResp = api
            .post("/api/v1/auth/login", &payload)
            .await
            .map_err(AppError::from)?;
        Ok(resp)
    }

    /// Request authentication challenge from backend
    ///
    /// # Arguments
    /// * `address` - User's Ethereum address
    ///
    /// # Returns
    /// SIWE message to be signed
    #[allow(dead_code)] // 用于 SIWE 认证流程
    pub async fn request_challenge(&self, address: &str) -> Result<SiweMessage, AppError> {
        let api = self.app_state.get_api_client();

        let response: ChallengeResponse = api
            .get(&format!("/api/v1/auth/challenge?address={}", address))
            .await?;

        let now = js_sys::Date::new_0();
        let issued_at = now.to_iso_string().as_string().unwrap();

        let expiration = js_sys::Date::new_0();
        expiration.set_time(now.get_time() + 600_000.0); // 10 minutes
        let expiration_time = expiration.to_iso_string().as_string().unwrap();

        Ok(SiweMessage {
            domain: "ironforge.wallet".to_string(),
            address: address.to_string(),
            statement: "Sign in to IronForge Wallet".to_string(),
            uri: api.base_url().to_string(),
            version: "1".to_string(),
            chain_id: 1, // Ethereum mainnet
            nonce: response.nonce,
            issued_at,
            expiration_time: Some(expiration_time),
            not_before: None,
        })
    }

    /// Sign SIWE message with wallet private key
    ///
    /// # Arguments
    /// * `message` - SIWE message
    /// * `key_manager` - User's key manager
    ///
    /// # Returns
    /// Signature (0x-prefixed hex string)
    #[allow(dead_code)] // 用于 SIWE 认证流程
    pub fn sign_message(
        &self,
        message: &SiweMessage,
        key_manager: &KeyManager,
    ) -> Result<String, AppError> {
        let message_str = message.to_string();
        let message_bytes = message_str.as_bytes();

        // EIP-191 personal sign prefix
        let prefixed_message = format!(
            "\x19Ethereum Signed Message:\n{}{}",
            message_bytes.len(),
            message_str
        );

        // Get the first Ethereum account's private key (index 0)
        let private_key_hex = key_manager.derive_eth_private_key(0).map_err(|_e| {
            AppError::Security(crate::shared::error::SecurityError::EncryptionFailed)
        })?;

        // Sign with secp256k1 using KeyManager
        let signature_hex = crate::crypto::key_manager::KeyManager::sign_eth_message(
            &private_key_hex,
            prefixed_message.as_bytes(),
        )
        .map_err(|_e| AppError::Security(crate::shared::error::SecurityError::EncryptionFailed))?;

        // Decode hex string to bytes
        let mut signature_bytes = hex::decode(&signature_hex).map_err(|_e| {
            AppError::Security(crate::shared::error::SecurityError::EncryptionFailed)
        })?;

        // EIP-191 signature format: r (32 bytes) + s (32 bytes) + v (1 byte)
        // Add recovery ID (v) - typically 27 or 28 for Ethereum
        // For simplicity, we'll use 27 (can be adjusted based on chain_id)
        if signature_bytes.len() == 64 {
            signature_bytes.push(27); // v = 27 (recovery ID)
        }

        Ok(format!("0x{}", hex::encode(signature_bytes)))
    }

    /// Verify signature and obtain JWT token
    ///
    /// # Arguments
    /// * `message` - Original SIWE message
    /// * `signature` - User's signature
    ///
    /// # Returns
    /// JWT authentication token
    #[allow(dead_code)] // 用于 SIWE 认证流程
    pub async fn verify_signature(
        &self,
        message: &SiweMessage,
        signature: &str,
    ) -> Result<String, AppError> {
        let api = self.app_state.get_api_client();

        let payload = VerifyRequest {
            message: message.to_string(),
            signature: signature.to_string(),
        };

        // deserialize 方法已自动提取 data 字段
        // 后端返回的格式可能是 {code, message, data: {token: ...}} 或直接 {token: ...}
        #[derive(Debug, Deserialize)]
        struct VerifyData {
            token: String,
        }

        let response: VerifyData = api
            .post("/api/v1/auth/verify", &payload)
            .await
            .map_err(AppError::from)?;
        Ok(response.token)
    }

    /// Complete SIWE authentication flow
    ///
    /// # Arguments
    /// * `address` - User's Ethereum address
    /// * `key_manager` - User's key manager
    ///
    /// # Returns
    /// JWT token for authenticated API requests
    ///
    /// Note: Caller should store token in API client via app_state.api.write().set_bearer_token()
    #[allow(dead_code)] // 用于 SIWE 认证流程
    pub async fn authenticate(
        &self,
        address: &str,
        key_manager: &KeyManager,
    ) -> Result<String, AppError> {
        // Step 1: Request challenge
        let message = self.request_challenge(address).await?;

        // Step 2: Sign message
        let signature = self.sign_message(&message, key_manager)?;

        // Step 3: Verify and get token
        let token = self.verify_signature(&message, &signature).await?;

        // Note: Token should be stored by caller using:
        // app_state.api.write().set_bearer_token(token.clone();

        Ok(token)
    }

    /// Refresh access token using refresh token
    /// 使用刷新令牌刷新访问令牌
    #[allow(dead_code)]
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<RefreshTokenResp, AppError> {
        let api = self.app_state.get_api_client();
        let payload = RefreshTokenReq {
            refresh_token: refresh_token.to_string(),
        };
        let resp: RefreshTokenResp = api
            .post("/api/v1/auth/refresh", &payload)
            .await
            .map_err(AppError::from)?;
        Ok(resp)
    }

    /// Logout user and revoke refresh token
    /// 登出用户并撤销刷新令牌
    pub async fn logout(&self) -> Result<LogoutResp, AppError> {
        let api = self.app_state.get_api_client();
        let resp: LogoutResp = api
            .post("/api/v1/auth/logout", &serde_json::json!({}))
            .await
            .map_err(AppError::from)?;
        Ok(resp)
    }

    /// Change user password
    /// 修改用户密码
    #[allow(dead_code)]
    pub async fn change_password(
        &self,
        old_password: &str,
        new_password: &str,
        confirm_password: &str,
    ) -> Result<(), AppError> {
        let api = self.app_state.get_api_client();
        let payload = ChangePasswordReq {
            old_password: old_password.to_string(),
            new_password: new_password.to_string(),
            confirm_password: confirm_password.to_string(),
        };
        // deserialize 方法已自动提取 data 字段
        // 后端返回: {code: 0, message: "success", data: {}}
        let _: crate::shared::api::EmptyResponse = api
            .post("/api/v1/auth/change-password", &payload)
            .await
            .map_err(AppError::from)?;
        Ok(())
    }
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RefreshTokenReq {
    pub refresh_token: String,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RefreshTokenResp {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChangePasswordReq {
    pub old_password: String,
    pub new_password: String,
    pub confirm_password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogoutResp {
    pub message: String,
}

/// Hook for using auth service in components
/// 获取认证服务实例
///
/// 注意：此函数当前未使用，但保留用于未来扩展
#[allow(dead_code)]
pub fn use_auth_service() -> AuthService {
    let app_state = use_context::<AppState>();
    AuthService::new(app_state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_siwe_message_formatting() {
        let message = SiweMessage {
            domain: "example.com".to_string(),
            address: "0x1234...5678".to_string(),
            statement: "Sign in to Example".to_string(),
            uri: "https://example.com".to_string(),
            version: "1".to_string(),
            chain_id: 1,
            nonce: "random_nonce".to_string(),
            issued_at: "2025-01-01T00:00:00Z".to_string(),
            expiration_time: None,
            not_before: None,
        };

        let formatted = message.to_string();
        assert!(formatted.contains("example.com"));
        assert!(formatted.contains("0x1234...5678"));
        assert!(formatted.contains("Sign in to Example"));
        assert!(formatted.contains("Nonce: random_nonce"));
    }

    #[test]
    fn test_message_with_expiration() {
        let message = SiweMessage {
            domain: "test.com".to_string(),
            address: "0xabc".to_string(),
            statement: "Test".to_string(),
            uri: "https://test.com".to_string(),
            version: "1".to_string(),
            chain_id: 1,
            nonce: "nonce".to_string(),
            issued_at: "2025-01-01T00:00:00Z".to_string(),
            expiration_time: Some("2025-01-01T01:00:00Z".to_string()),
            not_before: None,
        };

        let formatted = message.to_string();
        assert!(formatted.contains("Expiration Time: 2025-01-01T01:00:00Z"));
    }
}
