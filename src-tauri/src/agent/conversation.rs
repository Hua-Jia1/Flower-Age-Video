#![allow(dead_code)]
//! 对话管理器
//! 负责对话历史的存储、检索与上下文压缩

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::storage::Database;

/// 对话消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub role: String, // "user" | "assistant" | "system"
    pub content: String,
    pub timestamp: String,
}

/// 对话历史记录（存入 SQLite）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationHistory {
    pub id: String,
    pub project_id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub created_at: String,
}

/// 对话上下文（内存中）
#[derive(Debug, Clone)]
pub struct ConversationContext {
    pub session_id: String,
    pub project_id: String,
    /// 最近的消息（用于快速检索）
    pub recent_messages: VecDeque<ConversationMessage>,
    /// 已压缩的摘要（每 N 轮生成一次）
    pub summary: Option<String>,
    /// 历史消息总数（用于判断何时压缩）
    pub total_count: usize,
}

impl ConversationContext {
    pub fn new(session_id: String, project_id: String) -> Self {
        Self {
            session_id,
            project_id,
            recent_messages: VecDeque::with_capacity(20),
            summary: None,
            total_count: 0,
        }
    }

    /// 添加用户消息
    pub fn add_user_message(&mut self, content: String) {
        self.add_message("user".to_string(), content);
    }

    /// 添加助手消息
    pub fn add_assistant_message(&mut self, content: String) {
        self.add_message("assistant".to_string(), content);
    }

    fn add_message(&mut self, role: String, content: String) {
        self.recent_messages.push_back(ConversationMessage {
            role,
            content,
            timestamp: chrono::Utc::now().to_rfc3339(),
        });
        self.total_count += 1;

        // 超过容量时淘汰旧消息（但保留最后一条用于摘要）
        if self.recent_messages.len() > 20 {
            self.recent_messages.pop_front();
        }
    }

    /// 获取最近 N 条消息
    pub fn get_recent(&self, count: usize) -> Vec<ConversationMessage> {
        self.recent_messages.iter().rev().take(count).cloned().collect::<Vec<_>>().into_iter().rev().collect()
    }

    /// 判断是否需要压缩（每 10 轮压缩一次）
    pub fn needs_summarization(&self) -> bool {
        self.total_count > 0 && self.total_count % 10 == 0 && self.summary.is_none()
    }
}

/// 对话管理器
pub struct ConversationManager {
    db: Arc<Database>,
    /// 内存中的会话上下文（进程重启后从 DB 恢复）
    contexts: RwLock<Vec<ConversationContext>>,
    /// 摘要触发阈值（每 N 轮触发一次 LLM 摘要）
    summarization_threshold: usize,
    /// 最大保留消息数（压缩后）
    max_recent_messages: usize,
}

impl ConversationManager {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            contexts: RwLock::new(Vec::new()),
            summarization_threshold: 10,
            max_recent_messages: 20,
        }
    }

    /// 创建新会话
    pub async fn create_session(&self, session_id: String, project_id: String) -> ConversationContext {
        let context = ConversationContext::new(session_id.clone(), project_id.clone());

        // 持久化会话记录
        self.save_session_to_db(&session_id, &project_id).await;

        let mut contexts = self.contexts.write().await;
        contexts.push(context.clone());
        context
    }

    /// 获取或创建会话上下文
    pub async fn get_or_create_context(
        &self,
        session_id: &str,
        project_id: &str,
    ) -> ConversationContext {
        let contexts = self.contexts.read().await;
        if let Some(ctx) = contexts.iter().find(|c| c.session_id == session_id) {
            return ctx.clone();
        }
        drop(contexts);

        // 尝试从数据库恢复
        if let Some(history) = self.load_history_from_db(session_id).await {
            let mut ctx = ConversationContext::new(session_id.to_string(), project_id.to_string());
            for msg in history {
                ctx.add_message(msg.role, msg.content);
            }
            let mut contexts = self.contexts.write().await;
            contexts.push(ctx.clone());
            return ctx;
        }

        self.create_session(session_id.to_string(), project_id.to_string()).await
    }

    /// 添加用户消息
    pub async fn add_user_message(
        &self,
        session_id: &str,
        project_id: &str,
        content: String,
    ) -> ConversationContext {
        let mut contexts = self.contexts.write().await;
        if let Some(ctx) = contexts.iter_mut().find(|c| c.session_id == session_id) {
            ctx.add_user_message(content.clone());
            let ctx_clone = ctx.clone();
            drop(contexts);
            // 异步持久化
            let db = self.db.clone();
            let session_id = session_id.to_string();
            let project_id = project_id.to_string();
            tokio::spawn(async move {
                Self::persist_message_to_db(&db, &session_id, &project_id, "user", &content).await;
            });
            return ctx_clone;
        }
        drop(contexts);
        self.create_session(session_id.to_string(), project_id.to_string()).await
    }

    /// 添加助手消息
    pub async fn add_assistant_message(
        &self,
        session_id: &str,
        project_id: &str,
        content: String,
    ) -> ConversationContext {
        let mut contexts = self.contexts.write().await;
        if let Some(ctx) = contexts.iter_mut().find(|c| c.session_id == session_id) {
            ctx.add_assistant_message(content.clone());
            let ctx_clone = ctx.clone();
            drop(contexts);
            let db = self.db.clone();
            let session_id = session_id.to_string();
            let project_id = project_id.to_string();
            tokio::spawn(async move {
                Self::persist_message_to_db(&db, &session_id, &project_id, "assistant", &content).await;
            });
            return ctx_clone;
        }
        drop(contexts);
        self.create_session(session_id.to_string(), project_id.to_string()).await
    }

    /// 压缩历史（当 LLM 摘要生成后调用）
    pub async fn apply_summary(&self, session_id: &str, summary: String) {
        let mut contexts = self.contexts.write().await;
        if let Some(ctx) = contexts.iter_mut().find(|c| c.session_id == session_id) {
            ctx.summary = Some(summary);
            // 清空旧消息，只保留摘要
            ctx.recent_messages.clear();
        }
    }

    /// 获取用于 LLM 的上下文（带摘要压缩）
    pub async fn get_context_for_llm(
        &self,
        session_id: &str,
        max_messages: usize,
    ) -> Vec<ConversationMessage> {
        let contexts = self.contexts.read().await;
        if let Some(ctx) = contexts.iter().find(|c| c.session_id == session_id) {
            // 如果有摘要，加上系统消息
            let mut result = Vec::new();
            if let Some(ref summary) = ctx.summary {
                result.push(ConversationMessage {
                    role: "system".to_string(),
                    content: format!("[对话摘要] {}", summary),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                });
            }
            // 追加最近消息
            result.extend(ctx.get_recent(max_messages));
            return result;
        }
        Vec::new()
    }

    /// 获取当前会话的摘要文本
    pub async fn get_summary_text(&self, session_id: &str) -> Option<String> {
        let contexts = self.contexts.read().await;
        contexts.iter().find(|c| c.session_id == session_id).and_then(|c| c.summary.clone())
    }

    /// 判断是否需要生成摘要
    pub async fn needs_summarization(&self, session_id: &str) -> bool {
        let contexts = self.contexts.read().await;
        contexts.iter().find(|c| c.session_id == session_id).map(|c| c.needs_summarization()).unwrap_or(false)
    }

    // === 数据库操作 ===

    async fn save_session_to_db(&self, session_id: &str, project_id: &str) {
        if let Ok(conn) = self.db.conn.lock() {
            let now = chrono::Utc::now().to_rfc3339();
            let _ = conn.execute(
                "INSERT OR IGNORE INTO agent_sessions (id, project_id, user_input, status, result_json, created_at, updated_at) VALUES (?1, ?2, '', 'pending', '{}', ?3, ?3)",
                rusqlite::params![session_id, project_id, now],
            );
        }
    }

    async fn persist_message_to_db(db: &Arc<Database>, session_id: &str, project_id: &str, role: &str, content: &str) {
        if let Ok(conn) = db.conn.lock() {
            let now = chrono::Utc::now().to_rfc3339();
            let id = uuid::Uuid::new_v4().to_string();
            let _ = conn.execute(
                "INSERT INTO conversation_history (id, project_id, session_id, role, content, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![id, project_id, session_id, role, content, now],
            );
        }
    }

    async fn load_history_from_db(&self, session_id: &str) -> Option<Vec<ConversationHistory>> {
        if let Ok(conn) = self.db.conn.lock() {
            let mut stmt = conn
                .prepare("SELECT id, project_id, session_id, role, content, created_at FROM conversation_history WHERE session_id = ?1 ORDER BY created_at ASC")
                .ok()?;
            let rows = stmt
                .query_map(rusqlite::params![session_id], |row| {
                    Ok(ConversationHistory {
                        id: row.get(0)?,
                        project_id: row.get(1)?,
                        session_id: row.get(2)?,
                        role: row.get(3)?,
                        content: row.get(4)?,
                        created_at: row.get(5)?,
                    })
                })
                .ok()?;
            Some(rows.filter_map(|r| r.ok()).collect())
        } else {
            None
        }
    }
}

/// 新增 migration：创建 conversation_history 表
pub fn migration_add_conversation_history(db: &Database) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS conversation_history (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            session_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_conversation_session ON conversation_history(session_id);
        CREATE INDEX IF NOT EXISTS idx_conversation_project ON conversation_history(project_id);
        ",
    )
    .map_err(|e| e.to_string())
}