#![allow(dead_code)]
//! Gateway 请求/响应类型定义

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// API 请求类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiRequestType {
    Chat,
    ImageGen,
    VideoGen,
    Tts,
    MusicGen,
}

/// 聊天消息（用于 Chat 类型请求）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Gateway 请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayRequest {
    pub provider: String,
    pub model: String,
    pub request_type: ApiRequestType,
    pub messages: Option<Vec<ChatMessage>>,
    pub prompt: Option<String>,
    pub parameters: Value,
    pub max_retries: Option<u32>,
}

impl GatewayRequest {
    /// 创建 Chat 请求
    pub fn chat(provider: &str, model: &str, messages: Vec<ChatMessage>) -> Self {
        Self {
            provider: provider.to_string(),
            model: model.to_string(),
            request_type: ApiRequestType::Chat,
            messages: Some(messages),
            prompt: None,
            parameters: Value::Object(Default::default()),
            max_retries: Some(3),
        }
    }
}

/// Token 使用量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// Gateway 错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayError {
    pub code: String,
    pub message: String,
    pub retryable: bool,
}

/// Gateway 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayResponse {
    pub success: bool,
    pub data: Value,
    pub usage: Option<TokenUsage>,
    pub latency_ms: u64,
    pub error: Option<GatewayError>,
}

impl GatewayResponse {
    /// 创建成功响应
    pub fn ok(data: Value, usage: Option<TokenUsage>) -> Self {
        Self {
            success: true,
            data,
            usage,
            latency_ms: 0,
            error: None,
        }
    }

    /// 创建错误响应
    pub fn error(err: GatewayError) -> Self {
        Self {
            success: false,
            data: Value::Null,
            usage: None,
            latency_ms: 0,
            error: Some(err),
        }
    }
}
