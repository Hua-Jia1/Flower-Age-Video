#![allow(dead_code)]
//! 路由配置与请求校验

use super::types::GatewayRequest;
use std::collections::HashMap;

/// 路由配置
#[derive(Debug, Clone)]
pub struct RouteConfig {
    pub default_models: HashMap<String, String>,
}

impl Default for RouteConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl RouteConfig {
    pub fn new() -> Self {
        let mut defaults = HashMap::new();
        defaults.insert("openai".to_string(), "gpt-4o".to_string());
        defaults.insert(
            "anthropic".to_string(),
            "claude-sonnet-4-20250514".to_string(),
        );
        Self {
            default_models: defaults,
        }
    }

    /// 获取供应商的默认模型
    pub fn default_model(&self, provider: &str) -> Option<&str> {
        self.default_models.get(provider).map(|s| s.as_str())
    }

    /// 验证请求是否有效
    pub fn validate_request(&self, request: &GatewayRequest) -> Result<(), String> {
        if request.provider.is_empty() {
            return Err("Provider is required".to_string());
        }
        if request.model.is_empty() {
            return Err("Model is required".to_string());
        }
        Ok(())
    }
}
