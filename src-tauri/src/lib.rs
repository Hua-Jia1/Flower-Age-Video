mod agent;
mod gateway;
mod vault;
mod storage;
#[allow(dead_code)]
mod canvas;
#[allow(dead_code)]
mod media;
#[allow(dead_code)]
mod plugin;
mod commands;

use std::sync::Arc;
use tauri::Manager;
use storage::Database;
use agent::{AgentRuntime, ToolRegistry, Orchestrator, SpLoader, VibeAgent};
use vault::KeyVault;
use gateway::Gateway;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // 获取应用数据目录
            let app_data_dir = app.path().app_data_dir().expect("failed to get app data dir");
            // 确保数据目录存在（master.key / db 都需要）
            let _ = std::fs::create_dir_all(&app_data_dir);
            let db_path = app_data_dir.join("flowerage.db");

            // 初始化数据库
            let db = Arc::new(Database::new(&db_path).expect("failed to initialize database"));
            app.manage(db.clone());

            // 初始化 KeyVault（开发阶段：明文存储 API Key）
            let vault = Arc::new(KeyVault::new(db.clone()));
            app.manage(vault.clone());

            // 初始化 Gateway（使用通用 Provider，内部处理所有类型请求）
            let gateway = Arc::new(Gateway::new(vault.clone()));
            app.manage(gateway.clone());

            // 初始化 Agent Runtime / 工具注册表 / SP 加载器 / 编排器
            let runtime = Arc::new(AgentRuntime::new());
            let tool_registry = Arc::new(ToolRegistry::with_default_tools());

            // 尝试加载 SP（资源目录），失败则置为 None（不阻塞启动）
            let sp_loader = app
                .path()
                .resource_dir()
                .ok()
                .map(|p| p.join("agents"))
                .and_then(|p| SpLoader::new(&p).ok())
                .map(Arc::new);

            let orchestrator = Arc::new(Orchestrator::new(
                runtime.clone(),
                tool_registry.clone(),
                sp_loader,
                Some(gateway.clone()),
            ));
            app.manage(orchestrator);

            // 初始化 Vibe Agent 组件
            let conversation_mgr = Arc::new(
                agent::ConversationManager::new(db.clone())
            );
            let intent_classifier = Arc::new(
                agent::IntentClassifier::new(gateway.clone())
            );
            let tool_executor = Arc::new(
                agent::ToolExecutor::new(gateway.clone())
            );

            let vibe_agent = Arc::new(VibeAgent::new(
                conversation_mgr,
                intent_classifier,
                tool_executor,
                gateway.clone(),
                db.clone(),
                agent::VibeAgentConfig::default(),
            ));
            app.manage(vibe_agent);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::create_project,
            commands::list_projects,
            commands::get_project,
            commands::update_project,
            commands::delete_project,
            commands::save_canvas,
            commands::load_canvas,
            commands::start_agent_session,
            commands::start_vibe_session,
            commands::get_session_status,
            commands::list_sessions,
            commands::cancel_session,
            commands::save_api_key,
            commands::delete_api_key,
            commands::list_api_keys,
            commands::test_api_key,
            commands::gateway_send,
            commands::create_agent,
            commands::list_agents,
            commands::get_agent,
            commands::update_agent,
            commands::delete_agent,
            commands::create_skill,
            commands::list_skills,
            commands::get_skill,
            commands::update_skill,
            commands::delete_skill,
            commands::assign_skill_to_agent,
            commands::remove_skill_from_agent,
            commands::get_skills_for_agent,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}