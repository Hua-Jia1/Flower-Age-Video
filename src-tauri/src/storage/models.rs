#![allow(dead_code)]
use serde::{Deserialize, Serialize};

/// 项目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
}

/// 画布节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasNode {
    pub id: String,
    pub project_id: String,
    pub node_type: String,
    pub position_x: f64,
    pub position_y: f64,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub data_json: String, // 节点数据以 JSON 字符串存储
    pub created_at: String,
}

/// 画布边（连线）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasEdge {
    pub id: String,
    pub project_id: String,
    pub source_id: String,
    pub target_id: String,
    pub edge_type: String,
}

/// 资产
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: String,
    pub project_id: String,
    pub asset_type: String,
    pub file_path: String,
    pub thumbnail_path: Option<String>,
    pub metadata_json: String,
    pub created_at: String,
}

/// Agent 会话
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSession {
    pub id: String,
    pub project_id: String,
    pub user_input: String,
    pub status: String, // pending, thinking, executing, complete, error
    pub result_json: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Agent 执行日志
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentLog {
    pub id: String,
    pub session_id: String,
    pub agent_name: String,
    pub event_type: String, // thinking, tool_call, result, error
    pub message: String,
    pub timestamp: String,
}

/// 记忆条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub project_id: String,
    pub category: String, // preference, anchor, knowledge
    pub key: String,
    pub value_json: String,
    pub created_at: String,
    pub updated_at: String,
}

/// API Key（加密存储）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: String,
    pub provider: String,
    pub label: String,
    pub encrypted_key: Vec<u8>,
    pub nonce: Vec<u8>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// API 调用记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiUsage {
    pub id: String,
    pub provider: String,
    pub model: String,
    pub tokens_input: u32,
    pub tokens_output: u32,
    pub latency_ms: u32,
    pub success: bool,
    pub created_at: String,
}

/// Agent 定义（主Agent或子Agent）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub agent_type: String, // "main" | "sub"
    pub system_prompt: String,
    pub description: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Skill 定义（文本模板，供主Agent调用子Agent用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub prompt_template: String, // 文本模板
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}
