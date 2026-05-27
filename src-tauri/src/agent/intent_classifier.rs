#![allow(dead_code)]
//! 意图分类器
//! 使用 LLM 轻量分类，只花几十 token 判断用户意图和提取参数

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::gateway::Gateway;

/// 短剧/创作相关的意图类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Intent {
    /// 生成剧本/故事
    GenerateScript {
        title: Option<String>,
        genre: Option<String>,
        duration: Option<String>,
    },
    /// 生成分镜/镜头列表
    GenerateStoryboard {
        script: Option<String>,
        shot_count: Option<u32>,
    },
    /// 生成图像/图片
    GenerateImage {
        description: String,
        style: Option<String>,
    },
    /// 生成视频
    GenerateVideo {
        description: Option<String>,
        duration: Option<String>,
    },
    /// 生成音频/配音
    GenerateAudio {
        text: Option<String>,
        voice: Option<String>,
    },
    /// 生成音乐/BGM
    GenerateMusic {
        genre: Option<String>,
        mood: Option<String>,
        duration: Option<String>,
    },
    /// 修改画布内容（节点、连线等）
    ModifyCanvas {
        action: String, // "add_node", "delete_node", "connect", "update"
        target: Option<String>,
    },
    /// 闲聊/通用问答
    Chat {
        topic: Option<String>,
    },
    /// 未知意图
    Unknown,
}

impl Intent {
    /// 从 LLM 返回的字符串解析为 Intent
    pub fn from_llm_response(response: &str) -> Self {
        let response = response.trim();
        if response.starts_with('{') {
            // 尝试解析 JSON
            if let Ok(parsed) = serde_json::from_str::<Value>(response) {
                return Self::from_parsed_json(&parsed);
            }
        }
        // 回退到关键词匹配
        Self::keyword_matching(response)
    }

    fn from_parsed_json(value: &Value) -> Self {
        let intent_type = value.get("intent").and_then(|v| v.as_str()).unwrap_or("unknown");
        match intent_type {
            "generate_script" => Intent::GenerateScript {
                title: value.get("title").and_then(|v| v.as_str().map(String::from)),
                genre: value.get("genre").and_then(|v| v.as_str().map(String::from)),
                duration: value.get("duration").and_then(|v| v.as_str().map(String::from)),
            },
            "generate_storyboard" => Intent::GenerateStoryboard {
                script: value.get("script").and_then(|v| v.as_str().map(String::from)),
                shot_count: value.get("shot_count").and_then(|v| v.as_u64().map(|n| n as u32)),
            },
            "generate_image" => Intent::GenerateImage {
                description: value.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                style: value.get("style").and_then(|v| v.as_str().map(String::from)),
            },
            "generate_video" => Intent::GenerateVideo {
                description: value.get("description").and_then(|v| v.as_str().map(String::from)),
                duration: value.get("duration").and_then(|v| v.as_str().map(String::from)),
            },
            "generate_audio" => Intent::GenerateAudio {
                text: value.get("text").and_then(|v| v.as_str().map(String::from)),
                voice: value.get("voice").and_then(|v| v.as_str().map(String::from)),
            },
            "generate_music" => Intent::GenerateMusic {
                genre: value.get("genre").and_then(|v| v.as_str().map(String::from)),
                mood: value.get("mood").and_then(|v| v.as_str().map(String::from)),
                duration: value.get("duration").and_then(|v| v.as_str().map(String::from)),
            },
            "modify_canvas" => Intent::ModifyCanvas {
                action: value.get("action").and_then(|v| v.as_str()).unwrap_or("update").to_string(),
                target: value.get("target").and_then(|v| v.as_str().map(String::from)),
            },
            "chat" => Intent::Chat {
                topic: value.get("topic").and_then(|v| v.as_str().map(String::from)),
            },
            _ => Intent::Unknown,
        }
    }

    /// 关键词回退匹配
    fn keyword_matching(text: &str) -> Self {
        let text_lower = text.to_lowercase();
        if text_lower.contains("剧本") || text_lower.contains("故事") || text_lower.contains("段子") {
            Intent::GenerateScript { title: None, genre: None, duration: None }
        } else if text_lower.contains("分镜") || text_lower.contains("镜头") || text_lower.contains("shot") {
            Intent::GenerateStoryboard { script: None, shot_count: None }
        } else if text_lower.contains("图") || text_lower.contains("画") || text_lower.contains("image") || text_lower.contains("生成个") {
            // 提取描述部分（去掉"生成"、"图"等词）
            let desc = text.replace("生成", "").replace("图片", "").replace("图像", "").replace("画", "").trim().to_string();
            Intent::GenerateImage { description: desc, style: None }
        } else if text_lower.contains("视频") || text_lower.contains("video") || text_lower.contains("短片") {
            Intent::GenerateVideo { description: None, duration: None }
        } else if text_lower.contains("配音") || text_lower.contains("语音") || text_lower.contains("speech") {
            Intent::GenerateAudio { text: None, voice: None }
        } else if text_lower.contains("音乐") || text_lower.contains("歌") || text_lower.contains("bgm") || text_lower.contains("music") {
            Intent::GenerateMusic { genre: None, mood: None, duration: None }
        } else if text_lower.contains("画布") || text_lower.contains("节点") || text_lower.contains("连线") {
            Intent::ModifyCanvas { action: "update".to_string(), target: None }
        } else {
            Intent::Chat { topic: None }
        }
    }

    /// 获取意图的友好描述
    pub fn description(&self) -> &str {
        match self {
            Intent::GenerateScript { .. } => "生成剧本",
            Intent::GenerateStoryboard { .. } => "生成分镜",
            Intent::GenerateImage { .. } => "生成图片",
            Intent::GenerateVideo { .. } => "生成视频",
            Intent::GenerateAudio { .. } => "生成配音",
            Intent::GenerateMusic { .. } => "生成音乐",
            Intent::ModifyCanvas { .. } => "修改画布",
            Intent::Chat { .. } => "闲聊",
            Intent::Unknown { .. } => "未知意图",
        }
    }
}

/// 意图分类请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifyRequest {
    pub user_input: String,
    pub chat_history: Vec<super::orchestrator::ChatMessage>,
}

/// 意图分类响应
#[derive(Debug, Clone)]
pub struct ClassifyResult {
    pub intent: Intent,
    pub confidence: f32,
    pub reasoning: Option<String>,
}

/// 意图分类器
pub struct IntentClassifier {
    gateway: Arc<Gateway>,
}

impl IntentClassifier {
    pub fn new(gateway: Arc<Gateway>) -> Self {
        Self { gateway }
    }

    /// 使用 LLM 进行轻量分类（只花 ~50-100 token）
    pub async fn classify(&self, request: &ClassifyRequest) -> ClassifyResult {
        // 构建分类提示词
        let prompt = self.build_classify_prompt(&request.user_input);

        // 调用 LLM
        let llm_response = self.call_llm_classifier(&prompt).await;

        // 解析结果
        let intent = Intent::from_llm_response(&llm_response);
        let confidence = if matches!(intent, Intent::Unknown) { 0.3 } else { 0.85 };

        ClassifyResult {
            intent,
            confidence,
            reasoning: None,
        }
    }

    /// 使用关键词回退分类（无 LLM 时）
    pub fn classify_fallback(&self, user_input: &str) -> ClassifyResult {
        let intent = Intent::keyword_matching(user_input);
        ClassifyResult {
            intent,
            confidence: 0.6,
            reasoning: Some("关键词回退匹配".to_string()),
        }
    }

    fn build_classify_prompt(&self, user_input: &str) -> String {
        // 使用 JSON schema 让 LLM 返回结构化结果，减少 token 消耗
        r#"You are an intent classifier for a short drama creation AI assistant.
Classify the user input into ONE of these intents:
- generate_script: user wants to create a script/story/plot
- generate_storyboard: user wants to create storyboard/shots
- generate_image: user wants to generate an image
- generate_video: user wants to generate a video
- generate_audio: user wants to create voiceover/speech
- generate_music: user wants to create music/BGM
- modify_canvas: user wants to modify canvas (nodes, edges)
- chat: general chat/question

Extract key parameters as JSON:
{"intent": "...", "title": "...", "genre": "...", "description": "...", "duration": "...", "style": "...", "mood": "...", "voice": "...", "action": "..."}

User input: ""#.to_string() + user_input + r#""


Only return the JSON, no explanation. Keep it short."#
    }

    async fn call_llm_classifier(&self, prompt: &str) -> String {
        use crate::gateway::types::{ApiRequestType, ChatMessage as GwChatMessage, GatewayRequest};

        let request = GatewayRequest {
            provider: "llm".to_string(), // 从 vault 读取
            model: "".to_string(),
            request_type: ApiRequestType::Chat,
            messages: Some(vec![GwChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }]),
            prompt: None,
            parameters: serde_json::json!({
                "max_tokens": 100,
                "temperature": 0.1  // 低温度保证稳定输出
            }),
            max_retries: Some(1),
        };

        let resp = self.gateway.send(request).await;
        if resp.success {
            resp.data.get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()
        } else {
            // LLM 调用失败，返回空让回退生效
            String::new()
        }
    }
}

use std::sync::Arc;