#![allow(dead_code)]
//! 步数预算与降级矩阵

use serde::{Serialize, Deserialize};

/// 步数预算管理器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepBudget {
    pub max_steps: u32,
    pub consumed: u32,
}

impl StepBudget {
    pub fn new(max_steps: u32) -> Self {
        Self { max_steps, consumed: 0 }
    }

    /// 消耗一步，返回剩余步数
    pub fn consume(&mut self) -> Result<u32, String> {
        if self.is_exhausted() {
            return Err(format!("Budget exhausted: {}/{}", self.consumed, self.max_steps));
        }
        self.consumed += 1;
        Ok(self.remaining())
    }

    /// 剩余步数
    pub fn remaining(&self) -> u32 {
        self.max_steps.saturating_sub(self.consumed)
    }

    /// 是否已耗尽
    pub fn is_exhausted(&self) -> bool {
        self.consumed >= self.max_steps
    }

    /// 重置
    pub fn reset(&mut self) {
        self.consumed = 0;
    }

    /// 使用百分比
    pub fn usage_percent(&self) -> f32 {
        if self.max_steps == 0 { return 100.0; }
        (self.consumed as f32 / self.max_steps as f32) * 100.0
    }
}

/// 降级级别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegradationLevel {
    pub provider: String,
    pub model: String,
    pub max_retries: u32,
    pub current_retries: u32,
}

/// 降级矩阵 - 管理多级模型回退链
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegradationMatrix {
    levels: Vec<DegradationLevel>,
    current_level: usize,
}

impl DegradationMatrix {
    pub fn new(levels: Vec<DegradationLevel>) -> Self {
        Self { levels, current_level: 0 }
    }

    /// 获取默认的 Orchestrator 降级矩阵
    pub fn default_orchestrator() -> Self {
        Self::new(vec![
            DegradationLevel {
                provider: "openai".to_string(),
                model: "gpt-4o".to_string(),
                max_retries: 3,
                current_retries: 0,
            },
            DegradationLevel {
                provider: "anthropic".to_string(),
                model: "claude-sonnet-4-20250514".to_string(),
                max_retries: 3,
                current_retries: 0,
            },
            DegradationLevel {
                provider: "openai".to_string(),
                model: "gpt-4-turbo".to_string(),
                max_retries: 2,
                current_retries: 0,
            },
            DegradationLevel {
                provider: "openai".to_string(),
                model: "gpt-3.5-turbo".to_string(),
                max_retries: 2,
                current_retries: 0,
            },
        ])
    }

    /// 获取默认的子 Agent 降级矩阵
    pub fn default_sub_agent() -> Self {
        Self::new(vec![
            DegradationLevel {
                provider: "openai".to_string(),
                model: "gpt-4o".to_string(),
                max_retries: 2,
                current_retries: 0,
            },
            DegradationLevel {
                provider: "anthropic".to_string(),
                model: "claude-sonnet-4-20250514".to_string(),
                max_retries: 2,
                current_retries: 0,
            },
            DegradationLevel {
                provider: "openai".to_string(),
                model: "gpt-3.5-turbo".to_string(),
                max_retries: 1,
                current_retries: 0,
            },
        ])
    }

    /// 获取当前级别
    pub fn current(&self) -> Option<&DegradationLevel> {
        self.levels.get(self.current_level)
    }

    /// 记录失败并决定是否降级
    pub fn record_failure(&mut self) -> DegradationAction {
        if let Some(level) = self.levels.get_mut(self.current_level) {
            level.current_retries += 1;
            if level.current_retries >= level.max_retries {
                // 当前级别重试耗尽，降级到下一级
                self.current_level += 1;
                if self.current_level >= self.levels.len() {
                    return DegradationAction::AllExhausted;
                }
                return DegradationAction::Degraded;
            }
            DegradationAction::Retry
        } else {
            DegradationAction::AllExhausted
        }
    }

    /// 是否所有级别都已耗尽
    pub fn is_all_exhausted(&self) -> bool {
        self.current_level >= self.levels.len()
    }

    /// 重置矩阵
    pub fn reset(&mut self) {
        self.current_level = 0;
        for level in &mut self.levels {
            level.current_retries = 0;
        }
    }
}

/// 降级动作
#[derive(Debug, Clone, PartialEq)]
pub enum DegradationAction {
    Retry,         // 同级别重试
    Degraded,      // 降级到下一级
    AllExhausted,  // 所有级别耗尽
}
