#![allow(dead_code)]
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tera::{Tera, Context};

/// System Prompt 加载器
pub struct SpLoader {
    tera: Tera,
    base_path: PathBuf,
}

impl SpLoader {
    /// 创建 SP 加载器
    /// base_path 指向 resources/agents/ 目录
    pub fn new(base_path: &Path) -> Result<Self, String> {
        let pattern = base_path.join("*.md").to_string_lossy().to_string();
        let tera = Tera::new(&pattern).map_err(|e| format!("Failed to load SP templates: {}", e))?;

        Ok(Self {
            tera,
            base_path: base_path.to_path_buf(),
        })
    }

    /// 加载并渲染指定 Agent 的 SP
    pub fn load_sp(&self, agent_name: &str, variables: &HashMap<String, String>) -> Result<String, String> {
        let template_name = format!("{}.md", agent_name);

        let mut context = Context::new();
        for (key, value) in variables {
            context.insert(key, value);
        }

        self.tera
            .render(&template_name, &context)
            .map_err(|e| format!("Failed to render SP for '{}': {}", agent_name, e))
    }

    /// 列出所有可用的 Agent SP 名称
    pub fn list_agents(&self) -> Vec<String> {
        self.tera
            .get_template_names()
            .map(|name| name.trim_end_matches(".md").to_string())
            .collect()
    }

    /// 获取 SP 文件基础路径
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }

    /// 重新加载模板（热更新支持）
    pub fn reload(&mut self) -> Result<(), String> {
        let pattern = self.base_path.join("*.md").to_string_lossy().to_string();
        self.tera = Tera::new(&pattern).map_err(|e| format!("Failed to reload SP templates: {}", e))?;
        Ok(())
    }
}
