//! Anthropic Messages 适配器

use async_trait::async_trait;
use serde_json::Value;

use crate::gateway::provider::Provider;
use crate::gateway::types::{GatewayError, GatewayRequest, GatewayResponse, TokenUsage};

const ANTHROPIC_API_BASE: &str = "https://api.anthropic.com/v1";
const ANTHROPIC_VERSION: &str = "2023-06-01";

pub struct AnthropicProvider;

impl AnthropicProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AnthropicProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Provider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    fn supported_models(&self) -> Vec<&str> {
        vec![
            "claude-sonnet-4-20250514",
            "claude-3-haiku-20240307",
            "claude-3-5-sonnet-20241022",
        ]
    }

    async fn send_request(
        &self,
        request: &GatewayRequest,
        api_key: &str,
        client: &reqwest::Client,
    ) -> Result<GatewayResponse, GatewayError> {
        // Anthropic 将 system 消息单独提取
        let mut system_prompt = String::new();
        let mut messages: Vec<Value> = Vec::new();

        if let Some(msgs) = &request.messages {
            for msg in msgs {
                if msg.role == "system" {
                    system_prompt = msg.content.clone();
                } else {
                    messages.push(serde_json::json!({
                        "role": msg.role,
                        "content": msg.content,
                    }));
                }
            }
        }

        let mut body = serde_json::json!({
            "model": request.model,
            "messages": messages,
            "max_tokens": 4096,
        });

        if !system_prompt.is_empty() {
            if let Value::Object(ref mut obj) = body {
                obj.insert("system".to_string(), Value::String(system_prompt));
            }
        }

        // 合并额外参数（temperature 等），保护核心字段
        if let Value::Object(params) = &request.parameters {
            if let Value::Object(ref mut obj) = body {
                for (k, v) in params {
                    if k != "messages" && k != "model" {
                        obj.insert(k.clone(), v.clone());
                    }
                }
            }
        }

        // 发送请求（Anthropic 使用 x-api-key + anthropic-version 头）
        let response = client
            .post(format!("{}/messages", ANTHROPIC_API_BASE))
            .header("x-api-key", api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
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

        // 解析 Anthropic 响应：content[0].text
        let json: Value = serde_json::from_str(&response_text).map_err(|e| GatewayError {
            code: "PARSE_ERROR".to_string(),
            message: format!("Failed to parse response: {}", e),
            retryable: false,
        })?;

        let content = json["content"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let usage = json.get("usage").map(|u| TokenUsage {
            input_tokens: u["input_tokens"].as_u64().unwrap_or(0) as u32,
            output_tokens: u["output_tokens"].as_u64().unwrap_or(0) as u32,
        });

        Ok(GatewayResponse::ok(
            serde_json::json!({
                "content": content,
                "model": json["model"].as_str().unwrap_or(&request.model),
                "stop_reason": json["stop_reason"],
            }),
            usage,
        ))
    }
}
