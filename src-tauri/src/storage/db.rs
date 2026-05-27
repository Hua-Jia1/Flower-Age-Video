use rusqlite::{Connection, Result as SqlResult};
use std::path::Path;
use std::sync::Mutex;

use super::models::{Agent, Skill};

use uuid::Uuid;
use chrono::Utc;

/// 数据库封装
pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    /// 打开或创建数据库，并执行迁移
    pub fn new(db_path: &Path) -> SqlResult<Self> {
        // 确保父目录存在
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let conn = Connection::open(db_path)?;

        // 启用 WAL 模式（更好的并发性能）
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        // 启用外键约束
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;

        let db = Self {
            conn: Mutex::new(conn),
        };

        db.run_migrations()?;

        Ok(db)
    }

    /// 执行数据库迁移（建表）
    fn run_migrations(&self) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();

        let _ = conn.execute_batch("DROP TABLE IF EXISTS api_keys;");

        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS canvas_nodes (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                node_type TEXT NOT NULL,
                position_x REAL NOT NULL DEFAULT 0.0,
                position_y REAL NOT NULL DEFAULT 0.0,
                width REAL,
                height REAL,
                data_json TEXT NOT NULL DEFAULT '{}',
                created_at TEXT NOT NULL,
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS canvas_edges (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                source_id TEXT NOT NULL,
                target_id TEXT NOT NULL,
                edge_type TEXT NOT NULL DEFAULT 'smoothstep',
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS assets (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                asset_type TEXT NOT NULL,
                file_path TEXT NOT NULL,
                thumbnail_path TEXT,
                metadata_json TEXT NOT NULL DEFAULT '{}',
                created_at TEXT NOT NULL,
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS agent_sessions (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                user_input TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                result_json TEXT NOT NULL DEFAULT '{}',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS agent_logs (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                agent_name TEXT NOT NULL,
                event_type TEXT NOT NULL,
                message TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                FOREIGN KEY (session_id) REFERENCES agent_sessions(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS memory (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                category TEXT NOT NULL,
                key TEXT NOT NULL,
                value_json TEXT NOT NULL DEFAULT '{}',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_nodes_project ON canvas_nodes(project_id);
            CREATE INDEX IF NOT EXISTS idx_edges_project ON canvas_edges(project_id);
            CREATE INDEX IF NOT EXISTS idx_assets_project ON assets(project_id);
            CREATE INDEX IF NOT EXISTS idx_agent_sessions_project ON agent_sessions(project_id);
            CREATE INDEX IF NOT EXISTS idx_agent_logs_session ON agent_logs(session_id);
            CREATE INDEX IF NOT EXISTS idx_memory_project_category ON memory(project_id, category);

            -- 自定义供应商配置表（替代旧的 api_keys）
            CREATE TABLE IF NOT EXISTS custom_providers (
                id TEXT PRIMARY KEY,
                provider_type TEXT NOT NULL,
                label TEXT NOT NULL DEFAULT '',
                api_key TEXT NOT NULL,
                base_url TEXT NOT NULL DEFAULT '',
                model TEXT NOT NULL DEFAULT '',
                is_active INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE(provider_type, label)
            );
            CREATE INDEX IF NOT EXISTS idx_custom_providers_type ON custom_providers(provider_type);

            CREATE TABLE IF NOT EXISTS api_usage (
                id TEXT PRIMARY KEY,
                provider TEXT NOT NULL,
                model TEXT NOT NULL,
                tokens_input INTEGER NOT NULL DEFAULT 0,
                tokens_output INTEGER NOT NULL DEFAULT 0,
                latency_ms INTEGER NOT NULL DEFAULT 0,
                success INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_api_usage_provider ON api_usage(provider, created_at);

            /* Vibe Agent 对话历史表 */
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

            /* Agent 自定义表 */
            CREATE TABLE IF NOT EXISTS agents (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                agent_type TEXT NOT NULL DEFAULT 'sub',
                system_prompt TEXT NOT NULL DEFAULT '',
                description TEXT NOT NULL DEFAULT '',
                is_active INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS skills (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                prompt_template TEXT NOT NULL DEFAULT '',
                is_active INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS agent_skills (
                agent_id TEXT NOT NULL,
                skill_id TEXT NOT NULL,
                PRIMARY KEY (agent_id, skill_id)
            );
        ",
        )?;

        Ok(())
    }

    // === Agent CRUD ===

    pub fn create_agent(&self, name: &str, agent_type: &str, system_prompt: &str, description: &str) -> SqlResult<Agent> {
        let conn = self.conn.lock().unwrap();
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO agents (id, name, agent_type, system_prompt, description, is_active, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7)",
            rusqlite::params![id, name, agent_type, system_prompt, description, now, now],
        )?;

        Ok(Agent {
            id,
            name: name.to_string(),
            agent_type: agent_type.to_string(),
            system_prompt: system_prompt.to_string(),
            description: description.to_string(),
            is_active: true,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub fn list_agents(&self) -> SqlResult<Vec<Agent>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, agent_type, system_prompt, description, is_active, created_at, updated_at FROM agents ORDER BY created_at DESC")?;
        let rows = stmt.query_map([], |row| {
            Ok(Agent {
                id: row.get(0)?,
                name: row.get(1)?,
                agent_type: row.get(2)?,
                system_prompt: row.get(3)?,
                description: row.get(4)?,
                is_active: row.get::<_, i32>(5)? != 0,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;
        rows.collect()
    }

    pub fn get_agent(&self, id: &str) -> SqlResult<Option<Agent>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, agent_type, system_prompt, description, is_active, created_at, updated_at FROM agents WHERE id = ?1")?;
        let mut rows = stmt.query_map([id], |row| {
            Ok(Agent {
                id: row.get(0)?,
                name: row.get(1)?,
                agent_type: row.get(2)?,
                system_prompt: row.get(3)?,
                description: row.get(4)?,
                is_active: row.get::<_, i32>(5)? != 0,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    pub fn update_agent(&self, id: &str, name: &str, system_prompt: &str, description: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE agents SET name = ?1, system_prompt = ?2, description = ?3, updated_at = ?4 WHERE id = ?5",
            rusqlite::params![name, system_prompt, description, now, id],
        )?;
        Ok(())
    }

    pub fn delete_agent(&self, id: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM agents WHERE id = ?1", [id])?;
        Ok(())
    }

    // === Skill CRUD ===

    pub fn create_skill(&self, name: &str, description: &str, prompt_template: &str) -> SqlResult<Skill> {
        let conn = self.conn.lock().unwrap();
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO skills (id, name, description, prompt_template, is_active, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, 1, ?5, ?6)",
            rusqlite::params![id, name, description, prompt_template, now, now],
        )?;

        Ok(Skill {
            id,
            name: name.to_string(),
            description: description.to_string(),
            prompt_template: prompt_template.to_string(),
            is_active: true,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub fn list_skills(&self) -> SqlResult<Vec<Skill>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, description, prompt_template, is_active, created_at, updated_at FROM skills ORDER BY created_at DESC")?;
        let rows = stmt.query_map([], |row| {
            Ok(Skill {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                prompt_template: row.get(3)?,
                is_active: row.get::<_, i32>(4)? != 0,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?;
        rows.collect()
    }

    pub fn get_skill(&self, id: &str) -> SqlResult<Option<Skill>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, description, prompt_template, is_active, created_at, updated_at FROM skills WHERE id = ?1")?;
        let mut rows = stmt.query_map([id], |row| {
            Ok(Skill {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                prompt_template: row.get(3)?,
                is_active: row.get::<_, i32>(4)? != 0,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    pub fn update_skill(&self, id: &str, name: &str, description: &str, prompt_template: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE skills SET name = ?1, description = ?2, prompt_template = ?3, updated_at = ?4 WHERE id = ?5",
            rusqlite::params![name, description, prompt_template, now, id],
        )?;
        Ok(())
    }

    pub fn delete_skill(&self, id: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM skills WHERE id = ?1", [id])?;
        Ok(())
    }

    // === agent_skills 关联 ===

    pub fn assign_skill_to_agent(&self, agent_id: &str, skill_id: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO agent_skills (agent_id, skill_id) VALUES (?1, ?2)",
            [agent_id, skill_id],
        )?;
        Ok(())
    }

    pub fn remove_skill_from_agent(&self, agent_id: &str, skill_id: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM agent_skills WHERE agent_id = ?1 AND skill_id = ?2",
            [agent_id, skill_id],
        )?;
        Ok(())
    }

    pub fn get_skills_for_agent(&self, agent_id: &str) -> SqlResult<Vec<Skill>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT s.id, s.name, s.description, s.prompt_template, s.is_active, s.created_at, s.updated_at
             FROM skills s
             INNER JOIN agent_skills a ON s.id = a.skill_id
             WHERE a.agent_id = ?1"
        )?;
        let rows = stmt.query_map([agent_id], |row| {
            Ok(Skill {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                prompt_template: row.get(3)?,
                is_active: row.get::<_, i32>(4)? != 0,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?;
        rows.collect()
    }
}
