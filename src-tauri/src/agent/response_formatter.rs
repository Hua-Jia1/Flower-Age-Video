#![allow(dead_code)]
//! 响应格式化器
//! 负责将工具执行结果转换为自然语言回复和画布节点

use serde::{Deserialize, Serialize};

use super::tool_executor::ToolExecutionResult;
use super::orchestrator::GeneratedNode;

/// 响应格式化请求
#[derive(Debug, Clone)]
pub struct FormatRequest {
    /// 工具执行结果
    pub tool_results: Vec<ToolExecutionResult>,
    /// 原始用户输入
    pub user_input: String,
    /// 意图类型描述
    pub intent_description: String,
}

/// 格式化后的响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormattedResponse {
    /// 自然语言回复
    pub message: String,
    /// 生成的画布节点
    pub generated_nodes: Vec<GeneratedNode>,
    /// 执行摘要
    pub summary: String,
}

/// 响应格式化器
pub struct ResponseFormatter;

impl ResponseFormatter {
    pub fn new() -> Self {
        Self
    }

    /// 格式化工具执行结果
    pub fn format(&self, request: FormatRequest) -> FormattedResponse {
        let mut message = String::new();
        let mut generated_nodes = Vec::new();
        let mut summary_parts = Vec::new();

        for result in &request.tool_results {
            if result.success {
                match result.tool_name.as_str() {
                    "image" => {
                        let desc = result.output.get("description")
                            .and_then(|v| v.as_str())
                            .unwrap_or("图片");
                        message.push_str(&format!("已为你生成图片：{}\n", desc));

                        generated_nodes.push(GeneratedNode {
                            node_type: "image".to_string(),
                            label: "AI 生成图片".to_string(),
                            data: result.output.clone(),
                        });
                        summary_parts.push("图片生成");
                    }
                    "video" => {
                        let desc = result.output.get("description")
                            .and_then(|v| v.as_str())
                            .unwrap_or("视频");
                        message.push_str(&format!("已为你生成视频：{}\n", desc));

                        generated_nodes.push(GeneratedNode {
                            node_type: "video".to_string(),
                            label: "AI 生成视频".to_string(),
                            data: result.output.clone(),
                        });
                        summary_parts.push("视频生成");
                    }
                    "audio" => {
                        message.push_str("已为你生成配音。\n");
                        generated_nodes.push(GeneratedNode {
                            node_type: "audio".to_string(),
                            label: "AI 配音".to_string(),
                            data: result.output.clone(),
                        });
                        summary_parts.push("配音生成");
                    }
                    "music" => {
                        message.push_str("已为你生成背景音乐。\n");
                        generated_nodes.push(GeneratedNode {
                            node_type: "audio".to_string(),
                            label: "AI 背景音乐".to_string(),
                            data: result.output.clone(),
                        });
                        summary_parts.push("音乐生成");
                    }
                    "script" => {
                        let content = result.output.get("content")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        message.push_str(&format!("剧本已生成：\n{}\n", content));

                        generated_nodes.push(GeneratedNode {
                            node_type: "text".to_string(),
                            label: "短剧剧本".to_string(),
                            data: result.output.clone(),
                        });
                        summary_parts.push("剧本生成");
                    }
                    "storyboard" => {
                        message.push_str("分镜已生成，已为你添加到画布。\n");
                        generated_nodes.push(GeneratedNode {
                            node_type: "task".to_string(),
                            label: "分镜列表".to_string(),
                            data: result.output.clone(),
                        });
                        summary_parts.push("分镜生成");
                    }
                    "summarize" => {
                        // 摘要不生成节点
                    }
                    "chat" => {
                        // 闲聊回复直接使用 LLM 生成的内容
                        if let Some(content) = result.output.get("content").and_then(|v| v.as_str()) {
                            message.push_str(content);
                            summary_parts.push("对话");
                        }
                    }
                    _ => {
                        message.push_str(&format!("完成 {} 操作。\n", result.tool_name));
                    }
                }
            } else {
                message.push_str(&format!(
                    "{} 操作失败：{}\n",
                    result.tool_name,
                    result.error.as_deref().unwrap_or("未知错误")
                ));
            }
        }

        // 生成总结
        let summary = if summary_parts.is_empty() {
            format!("执行了 {} 个操作", request.tool_results.len())
        } else {
            summary_parts.join(" + ")
        };

        // 如果没有成功的结果，添加默认消息
        if message.is_empty() {
            message = format!("已处理你的请求：{}", request.user_input);
        }

        FormattedResponse {
            message,
            generated_nodes,
            summary,
        }
    }

    /// 生成错误回复
    pub fn format_error(&self, error: &str) -> FormattedResponse {
        FormattedResponse {
            message: format!("抱歉，出了点问题：{}", error),
            generated_nodes: vec![],
            summary: "执行失败".to_string(),
        }
    }

    /// 生成思考中回复
    pub fn format_thinking(&self, intent: &str) -> FormattedResponse {
        FormattedResponse {
            message: format!("好的，我来分析你的需求：{}...", intent),
            generated_nodes: vec![],
            summary: "正在分析".to_string(),
        }
    }

    /// 生成多步骤进度回复
    pub fn format_progress(&self, _step: &str, total: usize, current: usize) -> FormattedResponse {
        FormattedResponse {
            message: format!("[{}{}] {}/{}...", "●".repeat(current), "○".repeat(total - current), current, total),
            generated_nodes: vec![],
            summary: format!("进度 {}/{}", current, total),
        }
    }
}

impl Default for ResponseFormatter {
    fn default() -> Self {
        Self::new()
    }
}