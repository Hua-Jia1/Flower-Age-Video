#![allow(dead_code)]
use crate::storage::db::Database;
use crate::storage::models::{CanvasEdge, CanvasNode};
use rusqlite::params;

impl Database {
    /// 批量保存节点（全量替换——先删除该项目旧节点，再批量插入）
    pub fn save_nodes(&self, project_id: &str, nodes: &[CanvasNode]) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        // 先删除该项目的所有旧节点
        conn.execute(
            "DELETE FROM canvas_nodes WHERE project_id = ?1",
            params![project_id],
        )
        .map_err(|e| e.to_string())?;

        // 批量插入
        let mut stmt = conn
            .prepare(
                "INSERT INTO canvas_nodes (id, project_id, node_type, position_x, position_y, width, height, data_json, created_at) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            )
            .map_err(|e| e.to_string())?;

        for node in nodes {
            stmt.execute(params![
                node.id,
                project_id,
                node.node_type,
                node.position_x,
                node.position_y,
                node.width,
                node.height,
                node.data_json,
                node.created_at,
            ])
            .map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    /// 加载项目的所有节点
    pub fn load_nodes(&self, project_id: &str) -> Result<Vec<CanvasNode>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id, project_id, node_type, position_x, position_y, width, height, data_json, created_at 
                 FROM canvas_nodes WHERE project_id = ?1",
            )
            .map_err(|e| e.to_string())?;

        let nodes = stmt
            .query_map(params![project_id], |row| {
                Ok(CanvasNode {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    node_type: row.get(2)?,
                    position_x: row.get(3)?,
                    position_y: row.get(4)?,
                    width: row.get(5)?,
                    height: row.get(6)?,
                    data_json: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(nodes)
    }

    /// 批量保存边（全量替换）
    pub fn save_edges(&self, project_id: &str, edges: &[CanvasEdge]) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        // 先删除该项目的所有旧边
        conn.execute(
            "DELETE FROM canvas_edges WHERE project_id = ?1",
            params![project_id],
        )
        .map_err(|e| e.to_string())?;

        // 批量插入
        let mut stmt = conn
            .prepare(
                "INSERT INTO canvas_edges (id, project_id, source_id, target_id, edge_type) 
                 VALUES (?1, ?2, ?3, ?4, ?5)",
            )
            .map_err(|e| e.to_string())?;

        for edge in edges {
            stmt.execute(params![
                edge.id,
                project_id,
                edge.source_id,
                edge.target_id,
                edge.edge_type,
            ])
            .map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    /// 加载项目的所有边
    pub fn load_edges(&self, project_id: &str) -> Result<Vec<CanvasEdge>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id, project_id, source_id, target_id, edge_type 
                 FROM canvas_edges WHERE project_id = ?1",
            )
            .map_err(|e| e.to_string())?;

        let edges = stmt
            .query_map(params![project_id], |row| {
                Ok(CanvasEdge {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    source_id: row.get(2)?,
                    target_id: row.get(3)?,
                    edge_type: row.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(edges)
    }

    /// 删除单个节点（同时删除关联的边）
    pub fn delete_node(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM canvas_nodes WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        // 同时删除关联的边
        conn.execute(
            "DELETE FROM canvas_edges WHERE source_id = ?1 OR target_id = ?1",
            params![id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// 删除单条边
    pub fn delete_edge(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM canvas_edges WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
