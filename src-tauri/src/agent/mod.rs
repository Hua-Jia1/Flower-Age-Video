//! Agent 运行时模块
//! 负责 Agent 生命周期管理、工具调度、SP 注入

pub mod runtime;
pub mod orchestrator;
pub mod tools;
pub mod sp_loader;
pub mod budget;
pub mod conversation;
pub mod intent_classifier;
pub mod tool_executor;
pub mod response_formatter;
pub mod vibe_agent;

// Re-exports（仅导出外部使用的项）
pub use runtime::AgentRuntime;
pub use orchestrator::Orchestrator;
pub use tools::ToolRegistry;
pub use sp_loader::SpLoader;
pub use conversation::ConversationManager;
pub use intent_classifier::IntentClassifier;
pub use tool_executor::ToolExecutor;
pub use vibe_agent::{VibeAgent, VibeAgentConfig};