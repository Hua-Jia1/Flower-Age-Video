//! Agent 生命周期管理 - 状态机与会话管理

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::RwLock;
use chrono::Utc;
use uuid::Uuid;

/// Agent 状态枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentState {
    Idle,
    Thinking,
    Executing,
    Complete,
    Error,
    Cancelled,
}

/// Agent 配置（预留，子 Agent 启动参数）
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub name: String,
    pub max_steps: u32,
    pub sp_template: String,  // SP 模板名称
    pub tools: Vec<String>,   // 可用工具列表
}

/// Agent 实例（一次会话中的一个 Agent）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInstance {
    pub session_id: String,
    pub agent_name: String,
    pub state: AgentState,
    pub step_count: u32,
    pub max_steps: u32,
    pub logs: Vec<AgentLogEntry>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentLogEntry {
    pub event_type: String,
    pub message: String,
    pub timestamp: String,
}

/// Agent 运行时 - 管理所有 Agent 实例的生命周期
/// 使用 RwLock 允许多读少写场景下的高并发
pub struct AgentRuntime {
    sessions: RwLock<HashMap<String, AgentInstance>>,
}

impl AgentRuntime {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
        }
    }

    /// 创建新的 Agent 会话
    pub fn create_session(&self, agent_name: &str, max_steps: u32) -> String {
        let session_id = Uuid::new_v4().to_string();
        let instance = AgentInstance {
            session_id: session_id.clone(),
            agent_name: agent_name.to_string(),
            state: AgentState::Idle,
            step_count: 0,
            max_steps,
            logs: Vec::new(),
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        };
        self.sessions.write().unwrap().insert(session_id.clone(), instance);
        session_id
    }

    /// 获取会话状态
    pub fn get_session_status(&self, session_id: &str) -> Option<AgentInstance> {
        self.sessions.read().unwrap().get(session_id).cloned()
    }

    /// 更新会话状态
    pub fn update_state(&self, session_id: &str, new_state: AgentState) -> bool {
        if let Some(instance) = self.sessions.write().unwrap().get_mut(session_id) {
            instance.state = new_state;
            instance.updated_at = Utc::now().to_rfc3339();
            true
        } else {
            false
        }
    }

    /// 记录日志
    pub fn add_log(&self, session_id: &str, event_type: &str, message: &str) {
        if let Some(instance) = self.sessions.write().unwrap().get_mut(session_id) {
            instance.logs.push(AgentLogEntry {
                event_type: event_type.to_string(),
                message: message.to_string(),
                timestamp: Utc::now().to_rfc3339(),
            });
        }
    }

    /// 消耗一步
    #[allow(dead_code)]
    pub fn consume_step(&self, session_id: &str) -> Result<u32, String> {
        if let Some(instance) = self.sessions.write().unwrap().get_mut(session_id) {
            if instance.step_count >= instance.max_steps {
                return Err("Step budget exhausted".to_string());
            }
            instance.step_count += 1;
            instance.updated_at = Utc::now().to_rfc3339();
            Ok(instance.step_count)
        } else {
            Err("Session not found".to_string())
        }
    }

    /// 取消会话
    pub fn cancel_session(&self, session_id: &str) -> bool {
        self.update_state(session_id, AgentState::Cancelled)
    }

    /// 列出所有活跃会话
    #[allow(dead_code)]
    pub fn list_active_sessions(&self) -> Vec<AgentInstance> {
        self.sessions
            .read()
            .unwrap()
            .values()
            .filter(|s| matches!(s.state, AgentState::Thinking | AgentState::Executing))
            .cloned()
            .collect()
    }
}