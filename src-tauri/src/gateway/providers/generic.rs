//! 通用供应商适配器
//! 支持 LLM/Image/Video/Audio/Music 各类型请求，使用用户配置的自定义 base_url

use async_trait::async_trait;
use serde_json::Value;

use crate::gateway::provider::Provider;
use crate::gateway::types::{ApiRequestType, GatewayError, GatewayRequest, GatewayResponse, TokenUsage};

/// 通用 Provider（支持多类型请求）
pub struct GenericProvider;

impl GenericProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GenericProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Provider for GenericProvider {
    fn name(&self) -> &str {
        "generic"
    }

    fn supported_models(&self) -> Vec<&str> {
        vec!["*"]
    }

    async fn send_request(
        &self,
        request: &GatewayRequest,
        api_key: &str,
        client: &reqwest::Client,
    ) -> Result<GatewayResponse, GatewayError> {
        // 从 parameters 中读取 base_url
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
                message: "Provider requires a base_url to be configured".to_string(),
                retryable: false,
            });
        }

        match request.request_type {
            ApiRequestType::Chat => self.do_chat(request, &base_url, api_key, client).await,
            ApiRequestType::ImageGen => self.do_image_gen(request, &base_url, api_key, client).await,
            ApiRequestType::VideoGen => self.do_video_gen(request, &base_url, api_key, client).await,
            ApiRequestType::Tts => self.do_tts(request, &base_url, api_key, client).await,
            ApiRequestType::MusicGen => self.do_music_gen(request, &base_url, api_key, client).await,
        }
    }
}

impl GenericProvider {
    /// Chat 请求（OpenAI Chat Completions 兼容格式）
    async fn do_chat(
        &self,
        request: &GatewayRequest,
        base_url: &str,
        api_key: &str,
        client: &reqwest::Client,
    ) -> Result<GatewayResponse, GatewayError> {
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

        if let Value::Object(params) = &request.parameters {
            if let Value::Object(ref mut obj) = body {
                for (k, v) in params {
                    if !matches!(k.as_str(), "base_url" | "messages" | "model") {
                        obj.insert(k.clone(), v.clone());
                    }
                }
            }
        }

        let url = format!("{}/chat/completions", base_url);
        self.send_post(&url, body, api_key, client).await
    }

    /// 图片生成请求（DALL-E / 兼容格式）
    async fn do_image_gen(
        &self,
        request: &GatewayRequest,
        base_url: &str,
        api_key: &str,
        client: &reqwest::Client,
    ) -> Result<GatewayResponse, GatewayError> {
        let prompt = request.prompt.as_deref().unwrap_or("");
        let body = serde_json::json!({
            "model": request.model,
            "prompt": prompt,
        });

        // 尝试不同的图片生成端点
        let url = if base_url.contains("/v1") {
            format!("{}/images/generations", base_url)
        } else {
            format!("{}/v1/images/generations", base_url)
        };

        self.send_post(&url, body, api_key, client).await
    }

    /// 视频生成请求
    async fn do_video_gen(
        &self,
        request: &GatewayRequest,
        base_url: &str,
        api_key: &str,
        client: &reqwest::Client,
    ) -> Result<GatewayResponse, GatewayError> {
        let prompt = request.prompt.as_deref().unwrap_or("");
        let body = serde_json::json!({
            "model": request.model,
            "prompt": prompt,
        });

        let url = format!("{}/v1/video/generations", base_url);
        self.send_post(&url, body, api_key, client).await
    }

    /// TTS 请求
    async fn do_tts(
        &self,
        request: &GatewayRequest,
        base_url: &str,
        api_key: &str,
        client: &reqwest::Client,
    ) -> Result<GatewayResponse, GatewayError> {
        let prompt = request.prompt.as_deref().unwrap_or("");
        let voice = request
            .parameters
            .get("voice")
            .and_then(|v| v.as_str())
            .unwrap_or("alloy");

        let body = serde_json::json!({
            "model": request.model,
            "input": prompt,
            "voice": voice,
        });

        let url = format!("{}/v1/audio/speech", base_url);
        self.send_post(&url, body, api_key, client).await
    }

    /// 音乐生成请求
    async fn do_music_gen(
        &self,
        request: &GatewayRequest,
        base_url: &str,
        api_key: &str,
        client: &reqwest::Client,
    ) -> Result<GatewayResponse, GatewayError> {
        let prompt = request.prompt.as_deref().unwrap_or("");
        let body = serde_json::json!({
            "model": request.model,
            "prompt": prompt,
        });

        let url = format!("{}/v1/music/generations", base_url);
        self.send_post(&url, body, api_key, client).await
    }

    async fn send_post(
        &self,
        url: &str,
        body: Value,
        api_key: &str,
        client: &reqwest::Client,
    ) -> Result<GatewayResponse, GatewayError> {
        let response = client
            .post(url)
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

        // 尝试解析为 JSON
        if let Ok(json) = serde_json::from_str::<Value>(&response_text) {
            // 尝试提取 chat completions 格式的 content
            if let Some(content) = json["choices"][0]["message"]["content"].as_str() {
                let usage = json.get("usage").map(|u| TokenUsage {
                    input_tokens: u["prompt_tokens"].as_u64().unwrap_or(0) as u32,
                    output_tokens: u["completion_tokens"].as_u64().unwrap_or(0) as u32,
                });
                return Ok(GatewayResponse::ok(
                    serde_json::json!({
                        "content": content,
                        "model": json["model"].as_str().unwrap_or(""),
                        "finish_reason": json["choices"][0]["finish_reason"],
                    }),
                    usage,
                ));
            }
            // 其他 JSON 响应直接返回
            return Ok(GatewayResponse::ok(json, None));
        }

        // 非 JSON 响应直接返回原文
        Ok(GatewayResponse::ok(
            serde_json::json!({ "text": response_text }),
            None,
        ))
    }
}