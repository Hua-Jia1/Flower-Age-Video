use crate::storage::db::Database;
use crate::storage::models::Project;
use chrono::Utc;
use uuid::Uuid;
use rusqlite::params;

impl Database {
    /// 创建新项目
    pub fn create_project(&self, name: &str, description: &str) -> Result<Project, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO projects (id, name, description, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, name, description, now, now],
        ).map_err(|e| e.to_string())?;

        Ok(Project {
            id,
            name: name.to_string(),
            description: description.to_string(),
            created_at: now.clone(),
            updated_at: now,
        })
    }

    /// 列出所有项目
    pub fn list_projects(&self) -> Result<Vec<Project>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, created_at, updated_at FROM projects ORDER BY updated_at DESC"
        ).map_err(|e| e.to_string())?;

        let projects = stmt.query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        }).map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

        Ok(projects)
    }

    /// 获取单个项目
    pub fn get_project(&self, id: &str) -> Result<Option<Project>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, created_at, updated_at FROM projects WHERE id = ?1"
        ).map_err(|e| e.to_string())?;

        let mut rows = stmt.query_map(params![id], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        }).map_err(|e| e.to_string())?;

        match rows.next() {
            Some(Ok(project)) => Ok(Some(project)),
            Some(Err(e)) => Err(e.to_string()),
            None => Ok(None),
        }
    }

    /// 更新项目
    pub fn update_project(&self, id: &str, name: &str, description: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let now = Utc::now().to_rfc3339();

        conn.execute(
            "UPDATE projects SET name = ?1, description = ?2, updated_at = ?3 WHERE id = ?4",
            params![name, description, now, id],
        ).map_err(|e| e.to_string())?;

        Ok(())
    }

    /// 删除项目（级联删除节点和边）
    pub fn delete_project(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        // 由于建表时设置了 ON DELETE CASCADE，直接删除项目即可
        conn.execute("DELETE FROM projects WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
