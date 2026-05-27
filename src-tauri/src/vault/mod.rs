//! Key Vault 模块
//! API Key 本地明文存储（开发阶段，已移除加密）

pub mod encrypt;
pub mod store;

use std::sync::Arc;
use serde::{Serialize, Deserialize};
use crate::storage::db::Database;

pub use store::KeyStore;
// ProviderType 按需从 store 导入

/// 已存储密钥的信息（不含明文）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredKeyInfo {
    pub id: String,
    pub provider: String,
    pub label: String,
    pub base_url: String,
    pub model: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Key Vault — API 密钥存储（明文）
pub struct KeyVault {
    store: KeyStore,
}

impl KeyVault {
    /// 创建 KeyVault 实例
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            store: KeyStore::new(db),
        }
    }

    /// 存储 API Key（明文 + base_url + model）
    pub fn store_key(
        &self,
        provider: &str,
        label: &str,
        plain_key: &str,
        base_url: &str,
        model: &str,
    ) -> Result<(), String> {
        self.store.save_key(provider, label, plain_key, base_url, model)
    }

    /// 获取 API Key、base_url 与 model
    pub fn get_key(&self, provider: &str) -> Result<(String, String, String), String> {
        self.store.get_key(provider)
    }

    /// 删除 API Key
    pub fn delete_key(&self, provider: &str, label: &str) -> Result<(), String> {
        self.store.delete_key(provider, label)
    }

    /// 列出所有已存储的密钥信息（不含明文）
    pub fn list_keys(&self) -> Result<Vec<StoredKeyInfo>, String> {
        self.store.list_keys()
    }

    /// 检查某供应商是否有已配置的密钥
    pub fn has_key(&self, provider: &str) -> bool {
        self.store.get_key(provider).is_ok()
    }
}
