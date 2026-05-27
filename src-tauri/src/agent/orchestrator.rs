#![allow(dead_code)]
//! 主 Agent 编排器
//! 负责意图识别 → 任务拆解 → 派发 → 汇总

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

use super::runtime::{AgentRuntime, AgentState};
use super::sp_loader::SpLoader;
use super::tools::ToolRegistry;
use crate::gateway::Gateway;

// ─── 数据类型 ───────────────────────────────────────────────────────────────────

/// Agent 请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRequest {
    pub user_input: String,
    pub project_id: String,
    pub chat_history: Vec<ChatMessage>,
}

/// 聊天消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String, // "user" | "assistant" | "system"
    pub content: String,
}

/// Agent 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub session_id: String,
    pub status: String, // "thinking" | "executing" | "complete" | "error"
    pub progress: f32,  // 0.0 - 100.0
    pub summary: String,
    /// 完整的 AI 回复内容（自然语言）
    pub message: String,
    pub generated_nodes: Vec<GeneratedNode>,
}

/// 生成的画布节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedNode {
    pub node_type: String,
    pub label: String,
    pub data: Value,
}

/// 任务计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPlan {
    pub tasks: Vec<SubTask>,
}

/// 子任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubTask {
    pub agent_name: String,
    pub input: String,
    pub depends_on: Vec<String>, // 依赖的其他 agent_name
}

/// 子 Agent 执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubAgentResult {
    pub agent_name: String,
    pub success: bool,
    pub output: Value,
    pub error: Option<String>,
}

// ─── Orchestrator ───────────────────────────────────────────────────────────────

/// 主编排器
pub struct Orchestrator {
    pub runtime: Arc<AgentRuntime>,
    pub tool_registry: Arc<ToolRegistry>,
    pub sp_loader: Option<Arc<SpLoader>>, // Option 因为 SP 路径可能在初始化时不可用
    pub gateway: Option<Arc<Gateway>>,    // 可选 Gateway：可用且有 key 时走真实 API，否则 Mock 回退
}

impl Orchestrator {
    pub fn new(
        runtime: Arc<AgentRuntime>,
        tool_registry: Arc<ToolRegistry>,
        sp_loader: Option<Arc<SpLoader>>,
        gateway: Option<Arc<Gateway>>,
    ) -> Self {
        Self {
            runtime,
            tool_registry,
            sp_loader,
            gateway,
        }
    }

    /// 处理用户请求 - 主入口（Mock 实现）
    /// 模拟 Stage 1-3 流程，使用 tokio::time::sleep 模拟延迟
    pub async fn handle_request(&self, request: AgentRequest) -> AgentResponse {
        let session_id = self
            .runtime
            .create_session("flowerage-orchestrator", 400);

        // Stage 1: 意图理解
        self.runtime.update_state(&session_id, AgentState::Thinking);
        self.runtime
            .add_log(&session_id, "thinking", "分析用户意图...");


        let plan = self.understand_intent(&request.user_input).await;
        self.runtime.add_log(
            &session_id,
            "thinking",
            &format!("任务拆解完成：{} 个子任务", plan.tasks.len()),
        );

        // Stage 2: 派发子 Agent
        self.runtime
            .update_state(&session_id, AgentState::Executing);
        self.runtime
            .add_log(&session_id, "executing", "开始执行子任务...");


        let results = self.dispatch_agents(&plan).await;

        // Stage 3: 汇总
        self.runtime
            .add_log(&session_id, "executing", "汇总执行结果...");


        let response = self
            .aggregate_results(&session_id, &request, results)
            .await;
        self.runtime
            .update_state(&session_id, AgentState::Complete);
        self.runtime
            .add_log(&session_id, "result", &response.summary);

        response
    }

    /// Stage 1: 意图理解（Mock 实现）
    async fn understand_intent(&self, input: &str) -> TaskPlan {
        // Mock: 根据输入关键词简单路由
        let tasks = if input.contains("视频") || input.contains("video") {
            vec![
                SubTask {
                    agent_name: "image-agent".to_string(),
                    input: format!("为以下内容生成关键帧: {}", input),
                    depends_on: vec![],
                },
                SubTask {
                    agent_name: "video-agent".to_string(),
                    input: format!("基于关键帧生成视频: {}", input),
                    depends_on: vec!["image-agent".to_string()],
                },
            ]
        } else if input.contains("音乐") || input.contains("music") || input.contains("歌") {
            vec![SubTask {
                agent_name: "music-agent".to_string(),
                input: format!("创作音乐: {}", input),
                depends_on: vec![],
            }]
        } else if input.contains("配音") || input.contains("语音") || input.contains("speech") {
            vec![SubTask {
                agent_name: "speech-agent".to_string(),
                input: format!("生成语音: {}", input),
                depends_on: vec![],
            }]
        } else {
            // 默认：图像生成
            vec![SubTask {
                agent_name: "image-agent".to_string(),
                input: format!("生成图像: {}", input),
                depends_on: vec![],
            }]
        };

        TaskPlan { tasks }
    }

    /// Stage 2: 派发子 Agent
    /// 如果 Gateway 可用且默认 provider 已配置 key，则尝试调用真实 API；
    /// 否则回退到 Mock 实现。
    async fn dispatch_agents(&self, plan: &TaskPlan) -> Vec<SubAgentResult> {
        let mut results = Vec::new();

        for task in &plan.tasks {
            // 尝试使用 Gateway 调用真实 API（仅 chat 类任务 - image/video/music 类需专门适配，暂保留 Mock）
            if let Some(gateway) = self.gateway.as_ref() {
                // 尝试从 llm 配置中获取 key（provider_type = "llm"）
                if gateway.has_key_for("llm") {
                    use crate::gateway::types::{
                        ApiRequestType, ChatMessage as GwChatMessage, GatewayRequest,
                    };

                    // model 留空，从 vault 的 model 字段读取
                    let request = GatewayRequest {
                        provider: "llm".to_string(),
                        model: "".to_string(),
                        request_type: ApiRequestType::Chat,
                        messages: Some(vec![GwChatMessage {
                            role: "user".to_string(),
                            content: task.input.clone(),
                        }]),
                        prompt: None,
                        parameters: serde_json::json!({"max_tokens": 256}),
                        max_retries: Some(2),
                    };

                    let resp = gateway.send(request).await;
                    if resp.success {
                        results.push(SubAgentResult {
                            agent_name: task.agent_name.clone(),
                            success: true,
                            output: serde_json::json!({
                                "mock": false,
                                "agent": task.agent_name,
                                "provider": "llm",
                                "data": resp.data,
                                "usage": resp.usage,
                                "latency_ms": resp.latency_ms,
                            }),
                            error: None,
                        });
                        continue;
                    } else {
                        let err_msg = resp
                            .error
                            .as_ref()
                            .map(|e| format!("{}: {}", e.code, e.message))
                            .unwrap_or_else(|| "unknown gateway error".to_string());
                        results.push(SubAgentResult {
                            agent_name: task.agent_name.clone(),
                            success: false,
                            output: serde_json::json!({
                                "mock": false,
                                "agent": task.agent_name,
                                "provider": "llm",
                            }),
                            error: Some(err_msg),
                        });
                        continue;
                    }
                }
            }

            // 回退：Mock 实现
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            results.push(SubAgentResult {
                agent_name: task.agent_name.clone(),
                success: true,
                output: serde_json::json!({
                    "mock": true,
                    "agent": task.agent_name,
                    "message": format!("[Mock] {} 完成任务: {}", task.agent_name, task.input),
                }),
                error: None,
            });
        }

        results
    }

    /// Stage 3: 汇总结果
    async fn aggregate_results(
        &self,
        session_id: &str,
        request: &AgentRequest,
        results: Vec<SubAgentResult>,
    ) -> AgentResponse {
        let successful = results.iter().filter(|r| r.success).count();
        let total = results.len();

        let summary = format!(
            "已完成 {}/{} 个子任务。\n用户意图: {}\n[Mock 模式 - 实际 API 集成待配置]",
            successful, total, request.user_input
        );

        // Mock: 生成一个示例画布节点
        let generated_nodes = vec![GeneratedNode {
            node_type: "task".to_string(),
            label: format!(
                "Agent 任务: {}",
                &request.user_input[..request.user_input.len().min(20)]
            ),
            data: serde_json::json!({
                "status": "complete",
                "agents_used": results.iter().map(|r| r.agent_name.clone()).collect::<Vec<_>>(),
            }),
        }];

        AgentResponse {
            session_id: session_id.to_string(),
            status: "complete".to_string(),
            progress: 100.0,
            message: summary.clone(),
            summary,
            generated_nodes,
        }
    }
}
