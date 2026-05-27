use tauri::{State, Emitter};
use std::sync::Arc;
use crate::storage::Database;
use crate::storage::models::{Project, CanvasNode, CanvasEdge, Agent, Skill};
use serde::{Deserialize, Serialize};
use crate::agent::Orchestrator;
use crate::agent::orchestrator::{AgentRequest, AgentResponse, ChatMessage};
use crate::agent::VibeAgent;
use crate::vault::KeyVault;
use crate::gateway::Gateway;

/// 画布数据（前端传入/返回的格式）
#[derive(Debug, Serialize, Deserialize)]
pub struct CanvasData {
    pub nodes: Vec<CanvasNodeDto>,
    pub edges: Vec<CanvasEdgeDto>,
}

/// 前端节点 DTO（与 ReactFlow 格式对齐）
#[derive(Debug, Serialize, Deserialize)]
pub struct CanvasNodeDto {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: Option<String>,
    pub position: Position,
    pub data: serde_json::Value,
    pub width: Option<f64>,
    pub height: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

/// 前端边 DTO
#[derive(Debug, Serialize, Deserialize)]
pub struct CanvasEdgeDto {
    pub id: String,
    pub source: String,
    pub target: String,
    #[serde(rename = "type")]
    pub edge_type: Option<String>,
}

// === 项目命令 ===

#[tauri::command]
pub fn create_project(
    db: State<'_, Arc<Database>>,
    name: String,
    description: String,
) -> Result<Project, String> {
    db.create_project(&name, &description)
}

#[tauri::command]
pub fn list_projects(db: State<'_, Arc<Database>>) -> Result<Vec<Project>, String> {
    db.list_projects()
}

#[tauri::command]
pub fn get_project(db: State<'_, Arc<Database>>, id: String) -> Result<Option<Project>, String> {
    db.get_project(&id)
}

#[tauri::command]
pub fn update_project(
    db: State<'_, Arc<Database>>,
    id: String,
    name: String,
    description: String,
) -> Result<(), String> {
    db.update_project(&id, &name, &description)
}

#[tauri::command]
pub fn delete_project(db: State<'_, Arc<Database>>, id: String) -> Result<(), String> {
    db.delete_project(&id)
}

// === 画布命令 ===

#[tauri::command]
pub fn save_canvas(
    db: State<'_, Arc<Database>>,
    project_id: String,
    nodes: Vec<CanvasNodeDto>,
    edges: Vec<CanvasEdgeDto>,
) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();

    // DTO -> 存储模型
    let storage_nodes: Vec<CanvasNode> = nodes
        .into_iter()
        .map(|n| CanvasNode {
            id: n.id,
            project_id: project_id.clone(),
            node_type: n.node_type.unwrap_or_else(|| "text".to_string()),
            position_x: n.position.x,
            position_y: n.position.y,
            width: n.width,
            height: n.height,
            data_json: n.data.to_string(),
            created_at: now.clone(),
        })
        .collect();

    let storage_edges: Vec<CanvasEdge> = edges
        .into_iter()
        .map(|e| CanvasEdge {
            id: e.id,
            project_id: project_id.clone(),
            source_id: e.source,
            target_id: e.target,
            edge_type: e.edge_type.unwrap_or_else(|| "smoothstep".to_string()),
        })
        .collect();

    db.save_nodes(&project_id, &storage_nodes)?;
    db.save_edges(&project_id, &storage_edges)?;

    Ok(())
}

#[tauri::command]
pub fn load_canvas(
    db: State<'_, Arc<Database>>,
    project_id: String,
) -> Result<CanvasData, String> {
    let storage_nodes = db.load_nodes(&project_id)?;
    let storage_edges = db.load_edges(&project_id)?;

    // 存储模型 -> DTO
    let nodes: Vec<CanvasNodeDto> = storage_nodes
        .into_iter()
        .map(|n| CanvasNodeDto {
            id: n.id,
            node_type: Some(n.node_type),
            position: Position {
                x: n.position_x,
                y: n.position_y,
            },
            data: serde_json::from_str(&n.data_json)
                .unwrap_or(serde_json::Value::Object(Default::default())),
            width: n.width,
            height: n.height,
        })
        .collect();

    let edges: Vec<CanvasEdgeDto> = storage_edges
        .into_iter()
        .map(|e| CanvasEdgeDto {
            id: e.id,
            source: e.source_id,
            target: e.target_id,
            edge_type: Some(e.edge_type),
        })
        .collect();

    Ok(CanvasData { nodes, edges })
}

// === Agent 命令 ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStatusDto {
    pub session_id: String,
    pub agent_name: String,
    pub state: String,
    pub step_count: u32,
    pub max_steps: u32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummaryDto {
    pub session_id: String,
    pub project_id: String,
    pub user_input: String,
    pub status: String,
    pub created_at: String,
}

/// 启动 Agent 会话
#[tauri::command]
pub async fn start_agent_session(
    orchestrator: State<'_, Arc<Orchestrator>>,
    app_handle: tauri::AppHandle,
    project_id: String,
    user_input: String,
    chat_history: Vec<ChatMessage>,
) -> Result<AgentResponse, String> {
    let request = AgentRequest {
        user_input: user_input.clone(),
        project_id: project_id.clone(),
        chat_history,
    };

    // 发送 "thinking" 进度事件
    let _ = app_handle.emit(
        "agent-progress",
        serde_json::json!({
            "status": "thinking",
            "progress": 0.0,
            "message": "正在分析意图..."
        }),
    );

    let response = orchestrator.handle_request(request).await;

    // 发送 "complete" 进度事件
    let _ = app_handle.emit(
        "agent-progress",
        serde_json::json!({
            "status": "complete",
            "progress": 100.0,
            "message": &response.summary
        }),
    );

    Ok(response)
}

/// 启动 Vibe Agent 会话（对话式 AI）
#[tauri::command]
pub async fn start_vibe_session(
    vibe_agent: State<'_, Arc<VibeAgent>>,
    app_handle: tauri::AppHandle,
    project_id: String,
    user_input: String,
    chat_history: Vec<ChatMessage>,
) -> Result<AgentResponse, String> {
    let request = AgentRequest {
        user_input: user_input.clone(),
        project_id: project_id.clone(),
        chat_history,
    };

    // 发送 "thinking" 进度事件
    let _ = app_handle.emit(
        "agent-progress",
        serde_json::json!({
            "status": "thinking",
            "progress": 0.0,
            "message": "正在理解你的需求..."
        }),
    );

    let response = vibe_agent.handle(&request).await;

    // 发送 "complete" 进度事件
    let _ = app_handle.emit(
        "agent-progress",
        serde_json::json!({
            "status": "complete",
            "progress": 100.0,
            "message": &response.summary
        }),
    );

    Ok(response)
}

/// 获取会话状态
#[tauri::command]
pub async fn get_session_status(
    orchestrator: State<'_, Arc<Orchestrator>>,
    session_id: String,
) -> Result<SessionStatusDto, String> {
    let instance = orchestrator
        .runtime
        .get_session_status(&session_id)
        .ok_or_else(|| format!("Session not found: {}", session_id))?;

    Ok(SessionStatusDto {
        session_id: instance.session_id,
        agent_name: instance.agent_name,
        state: format!("{:?}", instance.state),
        step_count: instance.step_count,
        max_steps: instance.max_steps,
        created_at: instance.created_at,
        updated_at: instance.updated_at,
    })
}

/// 列出项目的会话历史
#[tauri::command]
pub async fn list_sessions(
    db: State<'_, Arc<Database>>,
    project_id: String,
) -> Result<Vec<SessionSummaryDto>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, project_id, user_input, status, created_at FROM agent_sessions WHERE project_id = ?1 ORDER BY created_at DESC LIMIT 50",
        )
        .map_err(|e| e.to_string())?;

    let sessions = stmt
        .query_map(rusqlite::params![project_id], |row| {
            Ok(SessionSummaryDto {
                session_id: row.get(0)?,
                project_id: row.get(1)?,
                user_input: row.get(2)?,
                status: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(sessions)
}

/// 取消 Agent 会话
#[tauri::command]
pub async fn cancel_session(
    orchestrator: State<'_, Arc<Orchestrator>>,
    app_handle: tauri::AppHandle,
    session_id: String,
) -> Result<(), String> {
    let success = orchestrator.runtime.cancel_session(&session_id);
    if !success {
        return Err(format!("Failed to cancel session: {}", session_id));
    }

    let _ = app_handle.emit(
        "agent-progress",
        serde_json::json!({
            "status": "cancelled",
            "progress": 0.0,
            "message": "会话已取消"
        }),
    );

    Ok(())
}

// === API Key 管理命令 ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyInfoDto {
    pub id: String,
    pub provider: String,
    pub label: String,
    pub base_url: String,
    pub model: String,
    pub is_active: bool,
    pub created_at: String,
}

/// 保存 API Key（provider_type: llm/image/video/audio/music）
#[tauri::command]
pub async fn save_api_key(
    vault: State<'_, Arc<KeyVault>>,
    provider: String,
    label: String,
    api_key: String,
    base_url: Option<String>,
    model: Option<String>,
) -> Result<(), String> {
    let base_url = base_url.unwrap_or_default();
    let model = model.unwrap_or_default();
    vault.store_key(&provider, &label, &api_key, &base_url, &model)
}

/// 删除 API Key
#[tauri::command]
pub async fn delete_api_key(
    vault: State<'_, Arc<KeyVault>>,
    provider: String,
    label: String,
) -> Result<(), String> {
    vault.delete_key(&provider, &label)
}

/// 列出所有 API Key（不返回明文）
#[tauri::command]
pub async fn list_api_keys(
    vault: State<'_, Arc<KeyVault>>,
) -> Result<Vec<ApiKeyInfoDto>, String> {
    let keys = vault.list_keys()?;
    Ok(keys
        .into_iter()
        .map(|k| ApiKeyInfoDto {
            id: k.id,
            provider: k.provider,
            label: k.label,
            base_url: k.base_url,
            model: k.model,
            is_active: k.is_active,
            created_at: k.created_at,
        })
        .collect())
}

/// 测试 API Key（发送最小化请求验证有效性）
#[tauri::command]
pub async fn test_api_key(
    gateway: State<'_, Arc<Gateway>>,
    provider: String,
    api_key: String,
    base_url: Option<String>,
    model: Option<String>,
    _request_type: Option<String>,
) -> Result<bool, String> {
    use crate::gateway::types::{ApiRequestType, ChatMessage as GwChatMessage, GatewayRequest};

    let base_url = base_url.unwrap_or_default();

    // 根据 provider_type 确定默认模型和请求类型
    let (default_model, req_type) = match provider.as_str() {
        "llm" => (
            model.as_deref().unwrap_or("gpt-3.5-turbo").to_string(),
            ApiRequestType::Chat,
        ),
        "image" => (
            model.as_deref().unwrap_or("dall-e-3").to_string(),
            ApiRequestType::ImageGen,
        ),
        "video" => (
            model.as_deref().unwrap_or("video-model").to_string(),
            ApiRequestType::VideoGen,
        ),
        "audio" => (
            model.as_deref().unwrap_or("tts-1").to_string(),
            ApiRequestType::Tts,
        ),
        "music" => (
            model.as_deref().unwrap_or("music-gen").to_string(),
            ApiRequestType::MusicGen,
        ),
        other => return Err(format!("Unknown provider type: {}", other)),
    };

    let mut params = serde_json::json!({"max_tokens": 5});
    if !base_url.is_empty() {
        if let serde_json::Value::Object(ref mut obj) = params {
            obj.insert("base_url".to_string(), serde_json::Value::String(base_url.clone()));
        }
    }

    let request = GatewayRequest {
        provider: provider.clone(),
        model: default_model,
        request_type: req_type,
        messages: Some(vec![GwChatMessage {
            role: "user".to_string(),
            content: "Hi".to_string(),
        }]),
        prompt: Some("Hi".to_string()),
        parameters: params,
        max_retries: Some(1),
    };

    match gateway.test_provider_key(&provider, &api_key, &base_url, &request).await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// 通过 Gateway 发送请求
#[tauri::command]
pub async fn gateway_send(
    gateway: State<'_, Arc<Gateway>>,
    provider: String,
    model: String,
    prompt: String,
) -> Result<serde_json::Value, String> {
    use crate::gateway::types::{ApiRequestType, ChatMessage as GwChatMessage, GatewayRequest};

    let request = GatewayRequest {
        provider,
        model,
        request_type: ApiRequestType::Chat,
        messages: Some(vec![GwChatMessage {
            role: "user".to_string(),
            content: prompt,
        }]),
        prompt: None,
        parameters: serde_json::json!({}),
        max_retries: Some(3),
    };

    let response = gateway.send(request).await;
    serde_json::to_value(&response).map_err(|e| e.to_string())
}

// === Agent 管理命令 ===

#[tauri::command]
pub fn create_agent(
    db: State<'_, Arc<Database>>,
    name: String,
    agent_type: String,
    system_prompt: String,
    description: String,
) -> Result<Agent, String> {
    db.create_agent(&name, &agent_type, &system_prompt, &description)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_agents(db: State<'_, Arc<Database>>) -> Result<Vec<Agent>, String> {
    db.list_agents().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_agent(db: State<'_, Arc<Database>>, id: String) -> Result<Option<Agent>, String> {
    db.get_agent(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_agent(
    db: State<'_, Arc<Database>>,
    id: String,
    name: String,
    system_prompt: String,
    description: String,
) -> Result<(), String> {
    db.update_agent(&id, &name, &system_prompt, &description)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_agent(db: State<'_, Arc<Database>>, id: String) -> Result<(), String> {
    db.delete_agent(&id).map_err(|e| e.to_string())
}

// === Skill 管理命令 ===

#[tauri::command]
pub fn create_skill(
    db: State<'_, Arc<Database>>,
    name: String,
    description: String,
    prompt_template: String,
) -> Result<Skill, String> {
    db.create_skill(&name, &description, &prompt_template)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_skills(db: State<'_, Arc<Database>>) -> Result<Vec<Skill>, String> {
    db.list_skills().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_skill(db: State<'_, Arc<Database>>, id: String) -> Result<Option<Skill>, String> {
    db.get_skill(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_skill(
    db: State<'_, Arc<Database>>,
    id: String,
    name: String,
    description: String,
    prompt_template: String,
) -> Result<(), String> {
    db.update_skill(&id, &name, &description, &prompt_template)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_skill(db: State<'_, Arc<Database>>, id: String) -> Result<(), String> {
    db.delete_skill(&id).map_err(|e| e.to_string())
}

// === Agent-Skill 关联命令 ===

#[tauri::command]
pub fn assign_skill_to_agent(
    db: State<'_, Arc<Database>>,
    agent_id: String,
    skill_id: String,
) -> Result<(), String> {
    db.assign_skill_to_agent(&agent_id, &skill_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_skill_from_agent(
    db: State<'_, Arc<Database>>,
    agent_id: String,
    skill_id: String,
) -> Result<(), String> {
    db.remove_skill_from_agent(&agent_id, &skill_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_skills_for_agent(
    db: State<'_, Arc<Database>>,
    agent_id: String,
) -> Result<Vec<Skill>, String> {
    db.get_skills_for_agent(&agent_id).map_err(|e| e.to_string())
}
