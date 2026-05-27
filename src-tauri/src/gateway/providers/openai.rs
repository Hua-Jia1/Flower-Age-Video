//! OpenAI Chat Completions 适配器

use async_trait::async_trait;
use serde_json::Value;

use crate::gateway::provider::Provider;
use crate::gateway::types::{GatewayError, GatewayRequest, GatewayResponse, TokenUsage};

const OPENAI_API_BASE: &str = "https://api.openai.com/v1";

pub struct OpenAiProvider;

impl OpenAiProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for OpenAiProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Provider for OpenAiProvider {
    fn name(&self) -> &str {
        "openai"
    }

    fn supported_models(&self) -> Vec<&str> {
        vec!["gpt-4o", "gpt-4-turbo", "gpt-3.5-turbo", "gpt-4o-mini"]
    }

    async fn send_request(
        &self,
        request: &GatewayRequest,
        api_key: &str,
        client: &reqwest::Client,
    ) -> Result<GatewayResponse, GatewayError> {
        // 构建消息数组
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

        // 合并额外参数（temperature, max_tokens 等）
        if let Value::Object(params) = &request.parameters {
            if let Value::Object(ref mut obj) = body {
                for (k, v) in params {
                    if k != "messages" && k != "model" {
                        obj.insert(k.clone(), v.clone());
                    }
                }
            }
        }

        // 发送请求
        let response = client
            .post(format!("{}/chat/completions", OPENAI_API_BASE))
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

        // 处理 HTTP 错误：429 与 5xx 可重试，4xx 其他不可重试
        if !status.is_success() {
            let retryable = status.as_u16() == 429 || status.as_u16() >= 500;
            return Err(GatewayError {
                code: format!("HTTP_{}", status.as_u16()),
                message: response_text,
                retryable,
            });
        }

        // 解析响应
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
