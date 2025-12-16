use crate::shared::error::ApiError;
use futures::future::{select, Either, FutureExt};
use futures::pin_mut;
use gloo_net::http::{Request, RequestBuilder, Response};
use gloo_timers::future::TimeoutFuture;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

/// 空响应类型（用于不需要返回数据的操作）
/// 后端返回: {code: 0, message: "success", data: {}}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmptyResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    _phantom: Option<()>,
}

#[derive(Clone, Debug)]
pub struct ApiConfig {
    pub base_url: String,
    pub timeout: u64,
}

impl Default for ApiConfig {
    fn default() -> Self {
        let base_url = option_env!("API_BASE_URL")
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or("http://localhost:8088")
            .to_string();

        Self {
            // IronCore backend default port 8088
            base_url,
            timeout: 30,
        }
    }
}

type RequestInterceptor = Arc<dyn Fn(&mut RequestBuilder) + Send + Sync>;
type ResponseInterceptor = Arc<dyn Fn(&Response) + Send + Sync>;

#[derive(Clone)]
pub struct ApiClient {
    config: ApiConfig,
    auth: Option<AuthToken>,
    request_interceptors: Arc<Vec<RequestInterceptor>>,
    response_interceptors: Arc<Vec<ResponseInterceptor>>,
}

#[derive(Clone)]
#[allow(dead_code)] // 用于 API 认证
enum AuthToken {
    ApiKey(String),
    Bearer(String),
}

impl ApiClient {
    pub fn new(config: ApiConfig) -> Self {
        Self {
            config,
            auth: None,
            request_interceptors: Arc::new(Vec::new()),
            response_interceptors: Arc::new(Vec::new()),
        }
    }

    #[allow(dead_code)] // 用于 API Key 认证
    pub fn set_api_key(&mut self, token: impl Into<String>) {
        self.auth = Some(AuthToken::ApiKey(token.into()));
    }

    pub fn set_bearer_token(&mut self, token: impl Into<String>) {
        self.auth = Some(AuthToken::Bearer(token.into()));
    }

    pub fn clear_auth(&mut self) {
        self.auth = None;
    }

    pub fn base_url(&self) -> &str {
        &self.config.base_url
    }

    #[allow(dead_code)] // 用于请求拦截器功能
    pub fn add_request_interceptor<F>(&mut self, interceptor: F)
    where
        F: Fn(&mut RequestBuilder) + Send + Sync + 'static,
    {
        let mut interceptors = (*self.request_interceptors).clone();
        interceptors.push(Arc::new(interceptor));
        self.request_interceptors = Arc::new(interceptors);
    }

    #[allow(dead_code)] // 用于响应拦截器功能
    pub fn add_response_interceptor<F>(&mut self, interceptor: F)
    where
        F: Fn(&Response) + Send + Sync + 'static,
    {
        let mut interceptors = (*self.response_interceptors).clone();
        interceptors.push(Arc::new(interceptor));
        self.response_interceptors = Arc::new(interceptors);
    }

    fn build_request(&self, method: &str, path: &str) -> RequestBuilder {
        let url = self.absolute_url(path);

        #[cfg(debug_assertions)]
        {
            use tracing::info;
            info!("🔍 API Request URL: {} {}", method, url);
            info!(
                "🔍 Path length: {}, Path bytes: {:?}",
                path.len(),
                path.as_bytes()
            );
            info!(
                "🔍 URL length: {}, URL bytes: {:?}",
                url.len(),
                url.as_bytes()
            );
        }

        let mut req = match method {
            "GET" => Request::get(&url),
            "POST" => Request::post(&url),
            "PUT" => Request::put(&url),
            "DELETE" => Request::delete(&url),
            "PATCH" => Request::patch(&url),
            _ => Request::get(&url), // Default to GET
        };

        if let Some(token) = &self.auth {
            req = match token {
                AuthToken::ApiKey(value) => req.header("X-API-Key", value),
                AuthToken::Bearer(value) => {
                    // Backend expects standard format: "Bearer <token>"
                    let header_val = format!("Bearer {}", value);
                    // Debug: Log token presence (but not the token itself for security)
                    #[cfg(debug_assertions)]
                    {
                        use tracing::debug;
                        debug!(
                            "API Request: Adding Authorization header (token length: {})",
                            value.len()
                        );
                    }
                    req.header("Authorization", &header_val)
                }
            };
        } else {
            #[cfg(debug_assertions)]
            {
                use tracing::warn;
                warn!(
                    "API Request: No auth token available for request to {}",
                    path
                );
            }
        }

        req = req
            .header("Content-Type", "application/json")
            .header("X-Client-Version", env!("CARGO_PKG_VERSION"))
            .header("X-Request-Id", Self::request_id().as_str())
            .header("X-Platform", "ironforge-web");

        for interceptor in self.request_interceptors.iter() {
            interceptor(&mut req);
        }
        req
    }

    async fn execute_with_retry(
        &self,
        method: &str,
        path: &str,
        body: Option<Value>,
    ) -> Result<Response, ApiError> {
        let mut attempts = 0;
        let max_attempts = 3;
        let mut delay_ms: u32 = 500; // Start with 500ms
        let timeout_ms = (self.config.timeout.saturating_mul(1000)).min(u32::MAX as u64) as u32;

        loop {
            let req_builder = self.build_request(method, path);
            let payload = body.clone();

            let send_future = async move {
                let response_result = if let Some(json_body) = payload {
                    req_builder
                        .json(&json_body)
                        .map_err(|e| {
                            #[cfg(debug_assertions)]
                            {
                                use tracing::error;
                                error!("API Request: Failed to serialize JSON body: {}", e);
                            }
                            ApiError::RequestFailed(e.to_string())
                        })?
                        .send()
                        .await
                } else {
                    req_builder.send().await
                };

                response_result.map_err(|e| {
                    #[cfg(debug_assertions)]
                    {
                        use tracing::error;
                        error!("API Request: Failed to send request: {}", e);
                    }
                    ApiError::RequestFailed(e.to_string())
                })
            };

            let resp_result = if timeout_ms == 0 {
                send_future.await
            } else {
                let timeout_future = TimeoutFuture::new(timeout_ms).map(|_| Err(ApiError::Timeout));
                pin_mut!(send_future);
                pin_mut!(timeout_future);
                match select(send_future, timeout_future).await {
                    Either::Left((res, _)) => res,
                    Either::Right((res, _)) => res,
                }
            };

            match resp_result {
                Ok(resp) => {
                    for interceptor in self.response_interceptors.iter() {
                        interceptor(&resp);
                    }
                    if resp.status() == 429 {
                        if attempts >= max_attempts {
                            return Err(ApiError::ResponseError("Rate limit exceeded".to_string()));
                        }

                        TimeoutFuture::new(delay_ms).await;

                        attempts += 1;
                        delay_ms = (delay_ms.saturating_mul(2)).min(8_000);
                        continue;
                    }
                    return Ok(resp);
                }
                Err(ApiError::Timeout) => {
                    if attempts >= max_attempts {
                        return Err(ApiError::Timeout);
                    }

                    TimeoutFuture::new(delay_ms).await;
                    attempts += 1;
                    delay_ms = (delay_ms.saturating_mul(2)).min(8_000);
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }
    }

    pub async fn request_json(
        &self,
        method: &str,
        path: &str,
        body: Option<Value>,
    ) -> Result<Value, ApiError> {
        let resp = self.execute_with_retry(method, path, body).await?;
        self.handle_json(resp).await
    }

    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, ApiError> {
        self.request_json("GET", path, None)
            .await
            .and_then(|value| Self::deserialize(value))
    }

    pub async fn post<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        let body_json =
            serde_json::to_value(body).map_err(|e| ApiError::RequestFailed(e.to_string()))?;
        self.request_json("POST", path, Some(body_json))
            .await
            .and_then(|value| Self::deserialize(value))
    }

    #[allow(dead_code)] // 用于 PUT 请求
    pub async fn put<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        let body_json =
            serde_json::to_value(body).map_err(|e| ApiError::RequestFailed(e.to_string()))?;
        self.request_json("PUT", path, Some(body_json))
            .await
            .and_then(|value| Self::deserialize(value))
    }

    #[allow(dead_code)] // 用于 DELETE 请求
    pub async fn delete<T: DeserializeOwned>(&self, path: &str) -> Result<T, ApiError> {
        self.request_json("DELETE", path, None)
            .await
            .and_then(|value| Self::deserialize(value))
    }

    async fn handle_json(&self, resp: Response) -> Result<Value, ApiError> {
        let status = resp.status();

        if !resp.ok() {
            if status == 401 {
                #[cfg(debug_assertions)]
                {
                    use tracing::warn;
                    warn!("API Response: 401 Unauthorized - Token may be expired or invalid");
                }
                return Err(ApiError::Unauthorized);
            }

            // Safely read response text, handle errors gracefully
            let text = match resp.text().await {
                Ok(t) => t,
                Err(e) => {
                    #[cfg(debug_assertions)]
                    {
                        use tracing::warn;
                        warn!("API Response: Failed to read error response body: {}", e);
                    }
                    format!("Failed to read response: {}", e)
                }
            };

            #[cfg(debug_assertions)]
            {
                use tracing::error;
                error!("API Response: Error {} - {}", status, text);
            }
            return Err(ApiError::ResponseError(format!("{} - {}", status, text)));
        }

        if status == 204 {
            return Ok(Value::Null);
        }

        // Safely parse JSON response
        match resp.json::<Value>().await {
            Ok(value) => Ok(value),
            Err(e) => {
                #[cfg(debug_assertions)]
                {
                    use tracing::error;
                    error!("API Response: Failed to parse JSON: {}", e);
                }
                Err(ApiError::ResponseError(format!(
                    "Failed to parse JSON response: {}",
                    e
                )))
            }
        }
    }

    fn deserialize<T: DeserializeOwned>(value: Value) -> Result<T, ApiError> {
        // 处理统一响应格式: { code, message, data }
        // 如果响应包含 "data" 字段，则提取 data 字段的内容
        if let Some(data) = value.get("data") {
            // 检查 code 字段，如果 code != 0，则返回错误
            if let Some(code) = value.get("code").and_then(|c| c.as_i64()) {
                if code != 0 {
                    let message = value
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("Unknown error")
                        .to_string();
                    return Err(ApiError::ResponseError(message));
                }
            }

            // 🔍 调试：打印 data 字段内容
            #[cfg(debug_assertions)]
            {
                use tracing::info;
                info!(
                    "📥 API Response data field: {}",
                    serde_json::to_string_pretty(data)
                        .unwrap_or_else(|_| "Failed to serialize".to_string())
                );
            }

            // 从 data 字段反序列化
            serde_json::from_value(data.clone()).map_err(|e| {
                #[cfg(debug_assertions)]
                {
                    use tracing::error;
                    error!("❌ Deserialization error: {}", e);
                    error!("   Expected type: {}", std::any::type_name::<T>());
                    error!(
                        "   Actual data: {}",
                        serde_json::to_string_pretty(data)
                            .unwrap_or_else(|_| "Failed to serialize".to_string())
                    );
                }
                ApiError::ResponseError(format!("Failed to deserialize data field: {}", e))
            })
        } else {
            // 如果没有 data 字段，直接反序列化整个响应（向后兼容）
            serde_json::from_value(value).map_err(|e| ApiError::ResponseError(e.to_string()))
        }
    }

    fn absolute_url(&self, path: &str) -> String {
        if path.starts_with("http://") || path.starts_with("https://") {
            path.to_string()
        } else {
            format!("{}{}", self.config.base_url.trim_end_matches('/'), path)
        }
    }

    fn request_id() -> String {
        let timestamp = js_sys::Date::new_0().get_time() as u64;
        let random = (js_sys::Math::random() * 1_000_000.0) as u64;
        format!("req-{}-{}", timestamp, random)
    }

    #[allow(dead_code)] // 用于获取认证令牌
    pub fn get_token(&self) -> Option<String> {
        match &self.auth {
            Some(AuthToken::ApiKey(t)) => Some(t.clone()),
            Some(AuthToken::Bearer(t)) => Some(t.clone()),
            None => None,
        }
    }
}
