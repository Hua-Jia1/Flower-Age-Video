//! API Gateway 模块
//! 本地 API 代理，路由到各 AI 供应商 API

pub mod types;
pub mod provider;
pub mod router;
pub mod rate_limiter;
pub mod providers;

use std::sync::Arc;
use serde_json::Value;

pub use types::{GatewayRequest, GatewayResponse, GatewayError};
pub use provider::Provider;
pub use rate_limiter::RateLimiter;
pub use providers::generic::GenericProvider;

use crate::vault::KeyVault;

/// API Gateway — 多供应商路由
pub struct Gateway {
    vault: Arc<KeyVault>,
    provider: Arc<GenericProvider>,
    rate_limiter: RateLimiter,
    http_client: reqwest::Client,
}

impl Gateway {
    pub fn new(vault: Arc<KeyVault>) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        Self {
            vault,
            provider: Arc::new(GenericProvider::new()),
            rate_limiter: RateLimiter::default(),
            http_client,
        }
    }

    /// 使用指定的明文 key 测试供应商连通性（绕过 vault）
    pub async fn test_provider_key(
        &self,
        provider_name: &str,
        api_key: &str,
        base_url: &str,
        request: &GatewayRequest,
    ) -> Result<GatewayResponse, GatewayError> {
        let mut params = request.parameters.clone();
        if let Value::Object(ref mut obj) = params {
            obj.insert("base_url".to_string(), Value::String(base_url.to_string()));
        } else {
            params = serde_json::json!({ "base_url": base_url });
        }

        let test_request = GatewayRequest {
            provider: provider_name.to_string(),
            model: request.model.clone(),
            request_type: request.request_type.clone(),
            messages: request.messages.clone(),
            prompt: request.prompt.clone(),
            parameters: params,
            max_retries: Some(1),
        };

        self.provider.send_request(&test_request, api_key, &self.http_client).await
    }

    /// 检查 vault 中是否存在指定供应商的 key
    pub fn has_key_for(&self, provider: &str) -> bool {
        self.vault.has_key(provider)
    }

    /// 获取 HTTP 客户端（供 provider 适配器共享）
    #[allow(dead_code)]
    pub fn http_client(&self) -> &reqwest::Client {
        &self.http_client
    }

    /// 发送请求 — 主入口
    pub async fn send(&self, mut request: GatewayRequest) -> GatewayResponse {
        // 1. 速率限制检查
        if let Err(e) = self.rate_limiter.acquire(&request.provider).await {
            return GatewayResponse::error(GatewayError {
                code: "RATE_LIMITED".to_string(),
                message: format!("Rate limit exceeded: {}", e),
                retryable: true,
            });
        }

        // 2. 获取 API Key、base_url 与 model
        let (api_key, base_url, model) = match self.vault.get_key(&request.provider) {
            Ok(triple) => triple,
            Err(e) => {
                return GatewayResponse::error(GatewayError {
                    code: "NO_API_KEY".to_string(),
                    message: format!("No API key configured for '{}': {}", request.provider, e),
                    retryable: false,
                });
            }
        };

        // 如果 model 非空，用它覆盖 request.model
        if !model.is_empty() {
            request.model = model;
        }

        // 注入 base_url 到 parameters
        if let Value::Object(ref mut obj) = request.parameters {
            obj.insert("base_url".to_string(), Value::String(base_url.clone()));
        } else {
            let mut map = serde_json::Map::new();
            map.insert("base_url".to_string(), Value::String(base_url.clone()));
            request.parameters = Value::Object(map);
        }

        // 3. 发送请求（统一使用 GenericProvider）
        let start = std::time::Instant::now();
        match self.provider.send_request(&request, &api_key, &self.http_client).await {
            Ok(mut response) => {
                response.latency_ms = start.elapsed().as_millis() as u64;
                response
            }
            Err(e) => GatewayResponse {
                success: false,
                data: serde_json::Value::Null,
                usage: None,
                latency_ms: start.elapsed().as_millis() as u64,
                error: Some(e),
            },
        }
    }

    /// 列出所有已注册的供应商（从 vault 动态获取去重列表）
    #[allow(dead_code)]
    pub async fn list_providers(&self) -> Vec<String> {
        let keys = self.vault.list_keys().unwrap_or_default();
        let mut providers: Vec<String> = keys
            .into_iter()
            .filter(|k| k.is_active)
            .map(|k| k.provider)
            .collect();
        providers.sort();
        providers.dedup();
        providers
    }

    /// 检查供应商是否已注册（查 vault 中是否有已激活的 key）
    #[allow(dead_code)]
    pub async fn has_provider(&self, name: &str) -> bool {
        self.vault.has_key(name)
    }
}

/// 兼容旧入口
#[allow(dead_code)]
pub fn init() {
    // 实际初始化由 Gateway::new() 完成
}