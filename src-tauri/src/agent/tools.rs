#![allow(dead_code)]
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// 工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters_schema: Value,
}

/// 工具执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: Value,
    pub error: Option<String>,
}

/// 工具处理器 trait
pub trait ToolHandler: Send + Sync {
    fn execute(&self, params: Value) -> ToolResult;
    fn name(&self) -> &str;
}

/// 工具注册表
pub struct ToolRegistry {
    definitions: HashMap<String, ToolDefinition>,
    handlers: HashMap<String, Arc<dyn ToolHandler>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            definitions: HashMap::new(),
            handlers: HashMap::new(),
        }
    }

    /// 创建预配置的主 Agent 工具注册表（Mock handlers）
    pub fn with_default_tools() -> Self {
        let mut registry = Self::new();

        // 注册 8 个主 Agent 工具
        let tools = vec![
            ("task_decompose", "将用户意图拆解为可执行的子任务列表", r#"{"type":"object","properties":{"input":{"type":"string"},"context":{"type":"object"}}}"#),
            ("agent_dispatch", "派发子 Agent 执行特定任务", r#"{"type":"object","properties":{"agent_name":{"type":"string"},"task":{"type":"string"},"depends_on":{"type":"array","items":{"type":"string"}}}}"#),
            ("canvas_read_nodes", "读取当前项目画布上的节点信息", r#"{"type":"object","properties":{"project_id":{"type":"string"},"node_type":{"type":"string"}}}"#),
            ("canvas_write_node", "向画布写入新节点", r#"{"type":"object","properties":{"project_id":{"type":"string"},"node_type":{"type":"string"},"position":{"type":"object"},"data":{"type":"object"}}}"#),
            ("canvas_update_edge", "更新画布连线关系", r#"{"type":"object","properties":{"project_id":{"type":"string"},"source_id":{"type":"string"},"target_id":{"type":"string"},"edge_type":{"type":"string"}}}"#),
            ("memory_read", "读取项目持久化 Memory", r#"{"type":"object","properties":{"project_id":{"type":"string"},"category":{"type":"string"},"key":{"type":"string"}}}"#),
            ("memory_write", "写入项目持久化 Memory", r#"{"type":"object","properties":{"project_id":{"type":"string"},"category":{"type":"string"},"key":{"type":"string"},"value":{"type":"object"}}}"#),
            ("output_summary", "生成用户可读的执行总结", r#"{"type":"object","properties":{"results":{"type":"array"},"format":{"type":"string"}}}"#),
        ];

        for (name, desc, schema) in tools {
            let schema_value: Value = serde_json::from_str(schema).unwrap_or(Value::Object(Default::default()));
            registry.register_definition(ToolDefinition {
                name: name.to_string(),
                description: desc.to_string(),
                parameters_schema: schema_value,
            });
            registry.register_handler(Arc::new(MockToolHandler { name: name.to_string() }));
        }

        registry
    }

    /// 注册工具定义
    pub fn register_definition(&mut self, def: ToolDefinition) {
        self.definitions.insert(def.name.clone(), def);
    }

    /// 注册工具处理器
    pub fn register_handler(&mut self, handler: Arc<dyn ToolHandler>) {
        self.handlers.insert(handler.name().to_string(), handler);
    }

    /// 获取工具定义
    pub fn get_definition(&self, name: &str) -> Option<&ToolDefinition> {
        self.definitions.get(name)
    }

    /// 列出所有工具定义
    pub fn list_tools(&self) -> Vec<&ToolDefinition> {
        self.definitions.values().collect()
    }

    /// 执行工具
    pub fn execute_tool(&self, name: &str, params: Value) -> ToolResult {
        match self.handlers.get(name) {
            Some(handler) => handler.execute(params),
            None => ToolResult {
                success: false,
                output: Value::Null,
                error: Some(format!("Tool not found: {}", name)),
            },
        }
    }

    /// 检查工具是否已注册
    pub fn has_tool(&self, name: &str) -> bool {
        self.definitions.contains_key(name)
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock 工具处理器（框架阶段占位）
struct MockToolHandler {
    name: String,
}

impl ToolHandler for MockToolHandler {
    fn execute(&self, params: Value) -> ToolResult {
        ToolResult {
            success: true,
            output: serde_json::json!({
                "mock": true,
                "tool": self.name,
                "message": format!("Mock execution of tool '{}' with params", self.name),
                "params_received": params,
            }),
            error: None,
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}
