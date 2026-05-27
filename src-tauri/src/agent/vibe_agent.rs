#![allow(dead_code)]
//! Vibe Coding Agent
//! 对话式 AI Agent，核心编排器

use std::sync::Arc;

use super::conversation::ConversationManager;
use super::intent_classifier::{ClassifyResult, Intent, IntentClassifier};
use super::tool_executor::{ToolCall, ToolExecutor};
use super::response_formatter::{FormatRequest, ResponseFormatter};
use super::orchestrator::{AgentRequest, AgentResponse};
use crate::gateway::Gateway;
use crate::storage::Database;
use crate::storage::models::{Agent as DbAgent, Skill as DbSkill};

/// VibeAgent 配置
pub struct VibeAgentConfig {
    /// 对话压缩阈值（每 N 轮压缩一次）
    pub summarization_threshold: usize,
    /// 最大保留消息数
    pub max_recent_messages: usize,
    /// 是否启用 LLM 分类（false 则用关键词回退）
    pub enable_llm_classification: bool,
}

impl Default for VibeAgentConfig {
    fn default() -> Self {
        Self {
            summarization_threshold: 10,
            max_recent_messages: 20,
            enable_llm_classification: true,
        }
    }
}

/// Vibe Coding Agent
pub struct VibeAgent {
    conversation: Arc<ConversationManager>,
    intent_classifier: Arc<IntentClassifier>,
    tool_executor: Arc<ToolExecutor>,
    response_formatter: Arc<ResponseFormatter>,
    gateway: Arc<Gateway>,
    db: Arc<Database>,
    config: VibeAgentConfig,
}

impl VibeAgent {
    pub fn new(
        conversation: Arc<ConversationManager>,
        intent_classifier: Arc<IntentClassifier>,
        tool_executor: Arc<ToolExecutor>,
        gateway: Arc<Gateway>,
        db: Arc<Database>,
        config: VibeAgentConfig,
    ) -> Self {
        Self {
            conversation,
            intent_classifier,
            tool_executor,
            response_formatter: Arc::new(ResponseFormatter::new()),
            gateway,
            db,
            config,
        }
    }

    /// 处理用户请求（主入口）
    pub async fn handle(&self, request: &AgentRequest) -> AgentResponse {
        let session_id = uuid::Uuid::new_v4().to_string();

        // 1. 记录用户消息到对话历史
        self.conversation
            .add_user_message(&session_id, &request.project_id, request.user_input.clone())
            .await;

        // 2. 获取对话上下文
        let _context = self.conversation
            .get_or_create_context(&session_id, &request.project_id)
            .await;

        // 3. 意图分类
        let classify_result = self.classify_intent(&request.user_input, &request.chat_history).await;

        // 4. 执行工具
        let tool_results = self.execute_according_to_intent(&classify_result.intent, &request.user_input, &request.chat_history).await;

        // 5. 生成助手回复
        let formatted = self.response_formatter.format(FormatRequest {
            tool_results,
            user_input: request.user_input.clone(),
            intent_description: classify_result.intent.description().to_string(),
        });

        // 6. 记录助手消息
        self.conversation
            .add_assistant_message(&session_id, &request.project_id, formatted.message.clone())
            .await;

        // 7. 检查是否需要压缩
        if self.conversation.needs_summarization(&session_id).await {
            self.trigger_summarization(&session_id).await;
        }

        // 8. 返回响应
        AgentResponse {
            session_id: session_id.clone(),
            status: "complete".to_string(),
            progress: 100.0,
            summary: formatted.summary.clone(),
            message: formatted.message.clone(),
            generated_nodes: formatted.generated_nodes,
        }
    }

    /// 意图分类
    async fn classify_intent(&self, user_input: &str, chat_history: &[super::orchestrator::ChatMessage]) -> ClassifyResult {
        if self.config.enable_llm_classification {
            self.intent_classifier
                .classify(&super::intent_classifier::ClassifyRequest {
                    user_input: user_input.to_string(),
                    chat_history: chat_history.to_vec(),
                })
                .await
        } else {
            self.intent_classifier.classify_fallback(user_input)
        }
    }

    /// 根据意图执行工具（从数据库读取 Skills 和 Agents）
    async fn execute_according_to_intent(&self, intent: &Intent, user_input: &str, chat_history: &[super::orchestrator::ChatMessage]) -> Vec<super::tool_executor::ToolExecutionResult> {
        // 闲聊意图：直接用 LLM 生成回复
        if matches!(intent, Intent::Chat { .. }) {
            let reply = self.generate_chat_response(user_input, chat_history).await;
            return vec![super::tool_executor::ToolExecutionResult {
                tool_name: "chat".to_string(),
                success: true,
                output: serde_json::json!({ "content": reply }),
                error: None,
                latency_ms: 0,
            }];
        }

        // 1. 读取数据库中的 Skills 和 子 Agents
        let skills = match self.db.list_skills() {
            Ok(s) => s.into_iter().filter(|s| s.is_active).collect::<Vec<_>>(),
            Err(_) => vec![],
        };
        let sub_agents = match self.db.list_agents() {
            Ok(agents) => agents.into_iter().filter(|a| a.is_active && a.agent_type == "sub").collect::<Vec<_>>(),
            Err(_) => vec![],
        };

        // 2. 匹配 Intent → Skill
        let matched_calls = self.match_intent_to_skill_calls(intent, &skills, &sub_agents);

        if matched_calls.is_empty() {
            vec![]
        } else {
            // 3. 执行工具调用
            self.tool_executor.execute_pipeline(matched_calls).await
        }
    }

    /// 使用 LLM 生成闲聊回复
    async fn generate_chat_response(&self, user_input: &str, chat_history: &[super::orchestrator::ChatMessage]) -> String {
        use crate::gateway::types::{ApiRequestType, ChatMessage as GwChatMessage, GatewayRequest};

        // 构建对话历史
        let mut messages: Vec<GwChatMessage> = vec![
            GwChatMessage {
                role: "system".to_string(),
                content: "你是一个友好的 AI 助手，负责与用户对话。你应该回答简洁、友好、自然。".to_string(),
            }
        ];

        // 添加历史消息（最多10条）
        for msg in chat_history.iter().rev().take(10) {
            messages.push(GwChatMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
            });
        }

        messages.push(GwChatMessage {
            role: "user".to_string(),
            content: user_input.to_string(),
        });

        let request = GatewayRequest {
            provider: "llm".to_string(),
            model: "".to_string(),
            request_type: ApiRequestType::Chat,
            messages: Some(messages),
            prompt: None,
            parameters: serde_json::json!({"max_tokens": 500}),
            max_retries: Some(2),
        };

        let resp = self.gateway.send(request).await;
        if resp.success {
            resp.data.get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("你好，有什么我可以帮你的吗？")
                .to_string()
        } else {
            "你好，有什么我可以帮你的吗？".to_string()
        }
    }

    /// 将 Intent 映射为 Skill/工具调用
    fn match_intent_to_skill_calls(&self, intent: &Intent, skills: &[DbSkill], _sub_agents: &[DbAgent]) -> Vec<ToolCall> {
        let intent_name = match intent {
            Intent::GenerateScript { .. } => "script",
            Intent::GenerateStoryboard { .. } => "storyboard",
            Intent::GenerateImage { .. } => "image",
            Intent::GenerateVideo { .. } => "video",
            Intent::GenerateAudio { .. } => "audio",
            Intent::GenerateMusic { .. } => "music",
            Intent::ModifyCanvas { .. } => "canvas",
            Intent::Chat { .. } => return vec![],
            Intent::Unknown => return vec![],
        };

        // 查找匹配的 Skill（通过 name 或 description 包含 intent_name）
        let matched_skill = skills.iter().find(|s| {
            s.name.to_lowercase().contains(intent_name) ||
            s.description.to_lowercase().contains(intent_name) ||
            s.prompt_template.to_lowercase().contains(intent_name)
        });

        // 从 Intent 提取参数
        let mut params = self.extract_intent_params(intent);

        // 如果找到匹配的 Skill，将其 prompt_template 注入到参数中
        if let Some(skill) = matched_skill {
            if let serde_json::Value::Object(ref mut obj) = params {
                obj.insert("skill_prompt".to_string(), 
                    serde_json::Value::String(skill.prompt_template.clone()));
                obj.insert("skill_name".to_string(), 
                    serde_json::Value::String(skill.name.clone()));
            }
        }

        // 构建工具调用
        vec![ToolCall {
            name: intent_name.to_string(),
            params,
            depends_on: vec![],
        }]
    }

    /// 从 Intent 提取参数
    fn extract_intent_params(&self, intent: &Intent) -> serde_json::Value {
        match intent {
            Intent::GenerateScript { title, genre, duration } => {
                serde_json::json!({
                    "title": title,
                    "genre": genre,
                    "duration": duration,
                })
            }
            Intent::GenerateStoryboard { script, shot_count } => {
                serde_json::json!({
                    "script": script,
                    "shot_count": shot_count,
                })
            }
            Intent::GenerateImage { description, style } => {
                serde_json::json!({
                    "description": description,
                    "style": style,
                })
            }
            Intent::GenerateVideo { description, duration } => {
                serde_json::json!({
                    "description": description,
                    "duration": duration,
                })
            }
            Intent::GenerateAudio { text, voice } => {
                serde_json::json!({
                    "text": text,
                    "voice": voice,
                })
            }
            Intent::GenerateMusic { genre, mood, duration } => {
                serde_json::json!({
                    "genre": genre,
                    "mood": mood,
                    "duration": duration,
                })
            }
            Intent::ModifyCanvas { action, target } => {
                serde_json::json!({
                    "action": action,
                    "target": target,
                })
            }
            Intent::Chat { .. } => serde_json::json!({}),
            Intent::Unknown => serde_json::json!({}),
        }
    }

    /// 触发上下文压缩
    async fn trigger_summarization(&self, session_id: &str) {
        let messages = self.conversation
            .get_context_for_llm(session_id, self.config.max_recent_messages)
            .await;

        if messages.is_empty() {
            return;
        }

        let params = serde_json::json!({
            "messages": messages.iter().map(|m| {
                serde_json::json!({
                    "role": m.role,
                    "content": m.content,
                })
            }).collect::<Vec<_>>()
        });

        let result = self.tool_executor.execute_tool(&ToolCall {
            name: "summarize".to_string(),
            params,
            depends_on: vec![],
        }).await;

        if result.success {
            if let Some(summary) = result.output.get("summary").and_then(|v| v.as_str()) {
                self.conversation.apply_summary(session_id, summary.to_string()).await;
            }
        }
    }
}