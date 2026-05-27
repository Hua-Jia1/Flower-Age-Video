//! 中转站（Relay）适配器
//! OpenAI Chat Completions 兼容格式，使用用户配置的自定义 base_url

use async_trait::async_trait;
use serde_json::Value;

use crate::gateway::provider::Provider;
use crate::gateway::types::{GatewayError, GatewayRequest, GatewayResponse, TokenUsage};

/// 中转站默认通用模型列表（仅用于 UI 提示，实际可发送任意模型名）
const DEFAULT_MODELS: &[&str] = &[
    "gpt-4o",
    "gpt-4-turbo",
    "gpt-3.5-turbo",
    "claude-sonnet-4-20250514",
];

pub struct RelayProvider;

impl RelayProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RelayProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Provider for RelayProvider {
    fn name(&self) -> &str {
        "relay"
    }

    fn supported_models(&self) -> Vec<&str> {
        DEFAULT_MODELS.to_vec()
    }

    async fn send_request(
        &self,
        request: &GatewayRequest,
        api_key: &str,
        client: &reqwest::Client,
    ) -> Result<GatewayResponse, GatewayError> {
        // 从 parameters 中读取 base_url（由 Gateway::send 注入）
        let base_url = request
            .parameters
            .get("base_url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .trim_end_matches('/')
            .to_string();

        if base_url.is_empty() {
            return Err(GatewayError {
                code: "MISSING_BASE_URL".to_string(),
                message: "Relay provider requires a base_url to be configured".to_string(),
                retryable: false,
            });
        }

        // 构建 OpenAI 兼容消息数组
        let empty_messages = Vec::new();
        let messages: Vec<Value> = request
            .messages
            .as_ref()
            .unwrap_or(&empty_messages)
            .iter()
            .map(|m| {
                serde_json::json!({
                    "role": m.role,
                    "content": m.content,
                })
            })
            .collect();

        let mut body = serde_json::json!({
            "model": request.model,
            "messages": messages,
        });

        // 合并额外参数（剔除 base_url 与保留字段）
        if let Value::Object(params) = &request.parameters {
            if let Value::Object(ref mut obj) = body {
                for (k, v) in params {
                    if k != "messages" && k != "model" && k != "base_url" {
                        obj.insert(k.clone(), v.clone());
                    }
                }
            }
        }

        let url = format!("{}/chat/completions", base_url);

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| GatewayError {
                code: "NETWORK_ERROR".to_string(),
                message: format!("Request failed: {}", e),
                retryable: true,
            })?;

        let status = response.status();
        let response_text = response.text().await.map_err(|e| GatewayError {
            code: "RESPONSE_READ_ERROR".to_string(),
            message: format!("Failed to read response: {}", e),
            retryable: false,
        })?;

        if !status.is_success() {
            let retryable = status.as_u16() == 429 || status.as_u16() >= 500;
            return Err(GatewayError {
                code: format!("HTTP_{}", status.as_u16()),
                message: response_text,
                retryable,
            });
        }

        let json: Value = serde_json::from_str(&response_text).map_err(|e| GatewayError {
            code: "PARSE_ERROR".to_string(),
            message: format!("Failed to parse response: {}", e),
            retryable: false,
        })?;

        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let usage = json.get("usage").map(|u| TokenUsage {
            input_tokens: u["prompt_tokens"].as_u64().unwrap_or(0) as u32,
            output_tokens: u["completion_tokens"].as_u64().unwrap_or(0) as u32,
        });

        Ok(GatewayResponse::ok(
            serde_json::json!({
                "content": content,
                "model": json["model"].as_str().unwrap_or(&request.model),
                "finish_reason": json["choices"][0]["finish_reason"],
            }),
            usage,
        ))
    }
}
