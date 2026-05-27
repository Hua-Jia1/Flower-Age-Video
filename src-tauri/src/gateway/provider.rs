#![allow(dead_code)]
//! 供应商适配器 trait
//!
//! 注意：使用 `async_trait` crate 以支持 `dyn Provider` trait object。
//! 需要在 Cargo.toml 中添加 `async-trait = "0.1"`。

use super::types::{GatewayError, GatewayRequest, GatewayResponse};
use async_trait::async_trait;

/// 供应商适配器 trait
#[async_trait]
pub trait Provider: Send + Sync {
    /// 供应商名称标识
    fn name(&self) -> &str;

    /// 支持的模型列表
    fn supported_models(&self) -> Vec<&str>;

    /// 发送请求到云端 API
    async fn send_request(
        &self,
        request: &GatewayRequest,
        api_key: &str,
        client: &reqwest::Client,
    ) -> Result<GatewayResponse, GatewayError>;
}
