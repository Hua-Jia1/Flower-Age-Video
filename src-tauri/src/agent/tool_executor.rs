#![allow(dead_code)]
//! 工具执行器
//! 负责并行/串行执行 AI 工具，并管理执行结果

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::gateway::Gateway;
use crate::gateway::types::{ApiRequestType, ChatMessage as GwChatMessage, GatewayRequest};

/// 工具调用请求
#[derive(Debug, Clone)]
pub struct ToolCall {
    /// 工具名称（image, video, audio, music, script, storyboard）
    pub name: String,
    /// 调用参数
    pub params: Value,
    /// 依赖的前置工具（等待其完成）
    pub depends_on: Vec<String>,
}

/// 工具执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionResult {
    pub tool_name: String,
    pub success: bool,
    pub output: Value,
    pub error: Option<String>,
    pub latency_ms: u64,
}

/// 工具执行器状态
#[derive(Debug, Clone)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// 工具执行器
pub struct ToolExecutor {
    gateway: Arc<Gateway>,
    /// 正在执行的任务
    running_tasks: RwLock<HashMap<String, Vec<ToolExecutionResult>>>,
}

impl ToolExecutor {
    pub fn new(gateway: Arc<Gateway>) -> Self {
        Self {
            gateway,
            running_tasks: RwLock::new(HashMap::new()),
        }
    }

    /// 执行单个工具调用
    pub async fn execute_tool(&self, call: &ToolCall) -> ToolExecutionResult {
        let start = std::time::Instant::now();

        match call.name.as_str() {
            "image" => self.generate_image(&call.params).await,
            "video" => self.generate_video(&call.params).await,
            "audio" => self.generate_audio(&call.params).await,
            "music" => self.generate_music(&call.params).await,
            "script" => self.generate_script(&call.params).await,
            "storyboard" => self.generate_storyboard(&call.params).await,
            "summarize" => self.summarize_context(&call.params).await,
            _ => ToolExecutionResult {
                tool_name: call.name.clone(),
                success: false,
                output: Value::Null,
                error: Some(format!("Unknown tool: {}", call.name)),
                latency_ms: start.elapsed().as_millis() as u64,
            },
        }
    }

    /// 执行多个工具（支持依赖）
    pub async fn execute_pipeline(&self, calls: Vec<ToolCall>) -> Vec<ToolExecutionResult> {
        let mut results: Vec<ToolExecutionResult> = Vec::new();
        let mut completed: HashMap<String, usize> = HashMap::new(); // tool_call_id -> result_index

        for (_idx, call) in calls.iter().enumerate() {
            // 检查依赖是否都已完成
            let deps_satisfied = call.depends_on.iter().all(|dep| {
                completed.contains_key(dep)
            });

            if !deps_satisfied {
                // 依赖未满足，标记为失败
                results.push(ToolExecutionResult {
                    tool_name: call.name.clone(),
                    success: false,
                    output: Value::Null,
                    error: Some("Dependencies not satisfied".to_string()),
                    latency_ms: 0,
                });
                continue;
            }

            // 收集依赖的输出作为上下文
            let context = call.depends_on.iter()
                .filter_map(|dep| completed.get(dep))
                .filter_map(|&i| results.get(i))
                .collect::<Vec<_>>();

            let result = self.execute_tool_with_context(call, &context).await;
            completed.insert(call.name.clone(), results.len());
            results.push(result);
        }

        results
    }

    /// 带上下文执行工具
    async fn execute_tool_with_context(
        &self,
        call: &ToolCall,
        context: &[&ToolExecutionResult],
    ) -> ToolExecutionResult {
        // 将上下文结果注入到参数中
        let mut params = call.params.clone();
        if !context.is_empty() {
            if let Value::Object(ref mut obj) = params {
                obj.insert("context".to_string(), serde_json::json!({
                    "dependencies": context.iter().map(|r| {
                        serde_json::json!({
                            "tool": r.tool_name,
                            "success": r.success,
                            "output": r.output,
                        })
                    }).collect::<Vec<_>>()
                }));
            }
        }
        self.execute_tool(&ToolCall {
            name: call.name.clone(),
            params,
            depends_on: vec![],
        }).await
    }

    // === 具体工具实现 ===

    /// 生成图像
    async fn generate_image(&self, params: &Value) -> ToolExecutionResult {
        let description = params.get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("生成一张图片");

        let style = params.get("style").and_then(|v| v.as_str());

        // 构建图像生成请求
        // 这里需要根据你的图像生成 API 调整
        let request = self.build_image_request(description, style);

        let start = std::time::Instant::now();
        let resp = self.gateway.send(request).await;

        ToolExecutionResult {
            tool_name: "image".to_string(),
            success: resp.success,
            output: resp.data,
            error: resp.error.map(|e| e.message),
            latency_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// 生成视频
    async fn generate_video(&self, params: &Value) -> ToolExecutionResult {
        let description = params.get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("生成一段视频");
        let duration = params.get("duration").and_then(|v| v.as_str());

        let request = self.build_video_request(description, duration);

        let start = std::time::Instant::now();
        let resp = self.gateway.send(request).await;

        ToolExecutionResult {
            tool_name: "video".to_string(),
            success: resp.success,
            output: resp.data,
            error: resp.error.map(|e| e.message),
            latency_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// 生成音频/配音
    async fn generate_audio(&self, params: &Value) -> ToolExecutionResult {
        let text = params.get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let voice = params.get("voice").and_then(|v| v.as_str());

        let request = self.build_audio_request(text, voice);

        let start = std::time::Instant::now();
        let resp = self.gateway.send(request).await;

        ToolExecutionResult {
            tool_name: "audio".to_string(),
            success: resp.success,
            output: resp.data,
            error: resp.error.map(|e| e.message),
            latency_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// 生成音乐
    async fn generate_music(&self, params: &Value) -> ToolExecutionResult {
        let genre = params.get("genre").and_then(|v| v.as_str());
        let mood = params.get("mood").and_then(|v| v.as_str());
        let duration = params.get("duration").and_then(|v| v.as_str());

        let request = self.build_music_request(genre, mood, duration);

        let start = std::time::Instant::now();
        let resp = self.gateway.send(request).await;

        ToolExecutionResult {
            tool_name: "music".to_string(),
            success: resp.success,
            output: resp.data,
            error: resp.error.map(|e| e.message),
            latency_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// 生成剧本
    async fn generate_script(&self, params: &Value) -> ToolExecutionResult {
        let title = params.get("title").and_then(|v| v.as_str());
        let genre = params.get("genre").and_then(|v| v.as_str());
        let duration = params.get("duration").and_then(|v| v.as_str());

        // 用 LLM 生成剧本
        let prompt = self.build_script_prompt(title, genre, duration, params);
        let request = GatewayRequest {
            provider: "llm".to_string(),
            model: "".to_string(), // 从 vault 中读取
            request_type: ApiRequestType::Chat,
            messages: Some(vec![GwChatMessage {
                role: "user".to_string(),
                content: prompt,
            }]),
            prompt: None,
            parameters: serde_json::json!({"max_tokens": 500}),
            max_retries: Some(2),
        };

        let start = std::time::Instant::now();
        let resp = self.gateway.send(request).await;

        ToolExecutionResult {
            tool_name: "script".to_string(),
            success: resp.success,
            output: if resp.success {
                resp.data.clone()
            } else {
                serde_json::json!({"error": resp.error.as_ref().map(|e| e.message.clone()).unwrap_or_default()})
            },
            error: resp.error.as_ref().map(|e| e.message.clone()),
            latency_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// 生成分镜
    async fn generate_storyboard(&self, params: &Value) -> ToolExecutionResult {
        let script = params.get("script")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let shot_count = params.get("shot_count")
            .and_then(|v| v.as_u64())
            .map(|n| n as u32);

        let prompt = self.build_storyboard_prompt(script, shot_count);
        let request = GatewayRequest {
            provider: "llm".to_string(),
            model: "".to_string(),
            request_type: ApiRequestType::Chat,
            messages: Some(vec![GwChatMessage {
                role: "user".to_string(),
                content: prompt,
            }]),
            prompt: None,
            parameters: serde_json::json!({"max_tokens": 500}),
            max_retries: Some(2),
        };

        let start = std::time::Instant::now();
        let resp = self.gateway.send(request).await;

        ToolExecutionResult {
            tool_name: "storyboard".to_string(),
            success: resp.success,
            output: if resp.success {
                resp.data.clone()
            } else {
                serde_json::json!({"error": resp.error.as_ref().map(|e| e.message.clone()).unwrap_or_default()})
            },
            error: resp.error.as_ref().map(|e| e.message.clone()),
            latency_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// 上下文摘要（用于压缩对话历史）
    async fn summarize_context(&self, params: &Value) -> ToolExecutionResult {
        let messages = params.get("messages")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|m| {
                        let role = m.get("role").and_then(|v| v.as_str()).unwrap_or("user");
                        let content = m.get("content").and_then(|v| v.as_str()).unwrap_or("");
                        Some(format!("{}: {}", role, content))
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap_or_default();

        let prompt = format!(
            "请简洁地总结以下对话的核心内容，保留关键信息和用户意图，生成100字以内的摘要：\n\n{}",
            messages
        );

        let request = GatewayRequest {
            provider: "llm".to_string(),
            model: "".to_string(),
            request_type: ApiRequestType::Chat,
            messages: Some(vec![GwChatMessage {
                role: "user".to_string(),
                content: prompt,
            }]),
            prompt: None,
            parameters: serde_json::json!({"max_tokens": 150}),
            max_retries: Some(1),
        };

        let start = std::time::Instant::now();
        let resp = self.gateway.send(request).await;

        ToolExecutionResult {
            tool_name: "summarize".to_string(),
            success: resp.success,
            output: serde_json::json!({
                "summary": if resp.success {
                    resp.data.get("content").and_then(|v| v.as_str()).unwrap_or("")
                } else {
                    "对话摘要生成失败"
                }
            }),
            error: resp.error.map(|e| e.message),
            latency_ms: start.elapsed().as_millis() as u64,
        }
    }

    // === 请求构建辅助 ===

    fn build_image_request(&self, description: &str, style: Option<&str>) -> GatewayRequest {
        GatewayRequest {
            provider: "image".to_string(),
            model: "".to_string(), // 从 vault 读取
            request_type: ApiRequestType::ImageGen,
            messages: Some(vec![GwChatMessage {
                role: "user".to_string(),
                content: if let Some(s) = style {
                    format!("{}，风格：{}", description, s)
                } else {
                    description.to_string()
                },
            }]),
            prompt: Some(description.to_string()),
            parameters: serde_json::json!({}),
            max_retries: Some(2),
        }
    }

    fn build_video_request(&self, description: &str, duration: Option<&str>) -> GatewayRequest {
        GatewayRequest {
            provider: "video".to_string(),
            model: "".to_string(), // 从 vault 读取
            request_type: ApiRequestType::VideoGen,
            messages: Some(vec![GwChatMessage {
                role: "user".to_string(),
                content: description.to_string(),
            }]),
            prompt: Some(description.to_string()),
            parameters: serde_json::json!({
                "duration": duration.unwrap_or("5s")
            }),
            max_retries: Some(1),
        }
    }

    fn build_audio_request(&self, text: &str, voice: Option<&str>) -> GatewayRequest {
        GatewayRequest {
            provider: "audio".to_string(),
            model: "".to_string(), // 从 vault 读取
            request_type: ApiRequestType::Tts,
            messages: Some(vec![GwChatMessage {
                role: "user".to_string(),
                content: text.to_string(),
            }]),
            prompt: Some(text.to_string()),
            parameters: serde_json::json!({
                "voice": voice.unwrap_or("alloy")
            }),
            max_retries: Some(2),
        }
    }

    fn build_music_request(&self, genre: Option<&str>, mood: Option<&str>, duration: Option<&str>) -> GatewayRequest {
        let mut content = String::from("生成一段");
        if let Some(g) = genre { content.push_str(g); content.push(' '); }
        if let Some(m) = mood { content.push_str(m); content.push(' '); }
        content.push_str("风格的背景音乐");

        GatewayRequest {
            provider: "music".to_string(),
            model: "".to_string(), // 从 vault 读取
            request_type: ApiRequestType::MusicGen,
            messages: Some(vec![GwChatMessage {
                role: "user".to_string(),
                content,
            }]),
            prompt: None,
            parameters: serde_json::json!({
                "duration": duration.unwrap_or("30s")
            }),
            max_retries: Some(1),
        }
    }

    fn build_script_prompt(&self, title: Option<&str>, genre: Option<&str>, duration: Option<&str>, params: &Value) -> String {
        let mut prompt = String::from("你是一个短剧剧本写作助手。请根据以下要求生成一个短剧剧本：\n\n");
        if let Some(t) = title {
            prompt.push_str(&format!("标题：{}\n", t));
        }
        if let Some(g) = genre {
            prompt.push_str(&format!("类型：{}\n", g));
        }
        if let Some(d) = duration {
            prompt.push_str(&format!("时长：{}\n", d));
        }

        // 额外参数
        if let Some(characters) = params.get("characters").and_then(|v| v.as_str()) {
            prompt.push_str(&format!("主要角色：{}\n", characters));
        }
        if let Some(plot) = params.get("plot").and_then(|v| v.as_str()) {
            prompt.push_str(&format!("剧情概要：{}\n", plot));
        }

        prompt.push_str("\n请以 JSON 格式输出剧本，包含：场景描述、人物对白、时长估算。格式如下：\n");
        prompt.push_str(r#"{"scenes": [{"description": "...", "dialogue": "...", "duration": "..."}]}"#);
        prompt
    }

    fn build_storyboard_prompt(&self, script: &str, shot_count: Option<u32>) -> String {
        let count = shot_count.unwrap_or(6);
        format!(
            r#"请将以下剧本拆分成 {} 个分镜，每个分镜包含：镜头编号、场景描述、画面内容、镜头类型（近景/远景/特写等）。

剧本：
{}

请以 JSON 数组格式输出：
[{{"shot": 1, "description": "...", "visual": "...", "type": "..."}}]"#,
            count, script
        )
    }
}