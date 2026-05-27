#![allow(dead_code)]
//! 密钥持久化存储（SQLite，明文存储）

use std::sync::Arc;
use chrono::Utc;
use uuid::Uuid;
use crate::storage::db::Database;
use super::StoredKeyInfo;

/// 供应商类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderType {
    Llm,
    Image,
    Video,
    Audio,
    Music,
    Custom,
}

impl ProviderType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Llm => "llm",
            Self::Image => "image",
            Self::Video => "video",
            Self::Audio => "audio",
            Self::Music => "music",
            Self::Custom => "custom",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "llm" => Self::Llm,
            "image" => Self::Image,
            "video" => Self::Video,
            "audio" => Self::Audio,
            "music" => Self::Music,
            _ => Self::Custom,
        }
    }
}

/// 密钥持久化存储
pub struct KeyStore {
    db: Arc<Database>,
}

impl KeyStore {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// 保存明文密钥（含 provider_type, base_url, model）
    pub fn save_key(
        &self,
        provider_type: &str,
        label: &str,
        plain_key: &str,
        base_url: &str,
        model: &str,
    ) -> Result<(), String> {
        let conn = self.db.conn.lock().map_err(|e| e.to_string())?;
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        conn.execute(
            "INSERT OR REPLACE INTO custom_providers (id, provider_type, label, api_key, base_url, model, is_active, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, ?8)",
            rusqlite::params![id, provider_type, label, plain_key, base_url, model, now, now],
        )
        .map_err(|e| format!("Failed to save key: {}", e))?;

        Ok(())
    }

    /// 获取明文密钥与配置（返回 api_key, base_url, model）
    pub fn get_key(&self, provider_type: &str) -> Result<(String, String, String), String> {
        let conn = self.db.conn.lock().map_err(|e| e.to_string())?;

        conn.query_row(
            "SELECT api_key, base_url, model FROM custom_providers WHERE provider_type = ?1 AND is_active = 1 LIMIT 1",
            rusqlite::params![provider_type],
            |row| {
                let api_key: String = row.get(0)?;
                let base_url: String = row.get(1)?;
                let model: String = row.get(2)?;
                Ok((api_key, base_url, model))
            },
        )
        .map_err(|e| format!("Key not found for provider '{}': {}", provider_type, e))
    }

    /// 删除密钥
    pub fn delete_key(&self, provider_type: &str, label: &str) -> Result<(), String> {
        let conn = self.db.conn.lock().map_err(|e| e.to_string())?;

        conn.execute(
            "DELETE FROM custom_providers WHERE provider_type = ?1 AND label = ?2",
            rusqlite::params![provider_type, label],
        )
        .map_err(|e| format!("Failed to delete key: {}", e))?;

        Ok(())
    }

    /// 列出所有密钥信息（不含明文）
    pub fn list_keys(&self) -> Result<Vec<StoredKeyInfo>, String> {
        let conn = self.db.conn.lock().map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare(
                "SELECT id, provider_type, label, base_url, model, is_active, created_at, updated_at FROM custom_providers ORDER BY provider_type, label",
            )
            .map_err(|e| e.to_string())?;

        let keys = stmt
            .query_map([], |row| {
                Ok(StoredKeyInfo {
                    id: row.get(0)?,
                    provider: row.get(1)?,
                    label: row.get(2)?,
                    base_url: row.get(3)?,
                    model: row.get(4)?,
                    is_active: row.get::<_, i32>(5)? != 0,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        Ok(keys)
    }
}
