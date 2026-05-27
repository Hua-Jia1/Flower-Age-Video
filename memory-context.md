# Flower Age Video — 项目记忆上下文

> 本文档汇总了项目开发过程中积累的所有记忆上下文，包括技术栈、环境配置、开发规范、重要决策、常见问题经验和任务总结。

---

## 一、项目介绍

### 1.1 短剧工作流与画布右键菜单规范
项目采用短剧工作流设计：以分镜为起点，串联图像生成、视频生成+配音、后期合成为端到端流水线；画布右键菜单仅支持内容创作节点（文本、图像、视频、音频、分镜、任务），禁用'项目'和'分组'等非创作类操作项。

### 1.2 API Gateway 功能概览
Phase 4 实现 API Gateway 多供应商 AI 路由层，核心能力包括：OpenAI 与 Anthropic 两个 LLM 供应商适配器、基于 AES-256-GCM 的密钥加密存储（Key Vault）、全局 + 供应商双维度令牌桶速率限制、Tauri IPC 命令暴露、Orchestrator 集成（支持 Mock 与真实调用无缝回退）、前端 Settings UI 密钥管理界面。

### 1.3 Agent 运行时框架功能概览
Phase 3 实现 Agent 运行时框架，核心能力包括：Agent 生命周期状态机（Idle/Thinking/Executing/Complete|Error）、MCP 风格工具注册与调度、基于步数的预算控制与多级模型降级矩阵、三阶段主编排逻辑（意图理解→子 Agent 派发→结果聚合），当前为 Mock 模式，真实 LLM 调用需后续配置 API Key。

---

## 二、技术栈

### 2.1 前端技术栈
- **前端框架**: React 19.2.6 + React DOM
- **构建工具**: Vite 8.0.13
- **类型系统**: TypeScript 6.0.3
- **样式方案**: Tailwind CSS 4.3.0 + shadcn 4.7.0 + tailwind-merge 3.6.0
- **UI 组件库**: @base-ui/react 1.4.1, lucide-react 1.16.0, framer-motion 12.39.0, @xyflow/react 12.10.2
- **桌面应用**: Tauri 2.11.x (@tauri-apps/api + @tauri-apps/cli)
- **字体**: @fontsource-variable/geist 5.2.9

### 2.2 Agent 运行时技术栈
项目采用 Tauri Event System（`app.emit` / `@tauri-apps/api/event`）作为 Agent 运行时通信机制，替代 HTTP SSE；LLM 调用在本阶段仅实现 Mock 层，不实际发起网络请求；新增依赖仅 `tera = "1"` 用于 SP 模板渲染。

### 2.3 API Gateway 技术栈变更
项目技术栈更新：移除 `ring` 和 `rand` 加密依赖库；中转站供应商需支持 OpenAI Chat Completions 标准接口（/v1/chat/completions）。

### 2.4 agent-progress 事件通信机制
项目采用 Tauri Event System 进行前端与 Rust Agent 编排器通信，其中 `agent-progress` 事件用于实时推送 agent 执行状态（如 thinking/executing/complete/error）至 ChatPanel UI 更新状态。

---

## 三、环境与构建配置

### 3.1 Tauri 运行环境启动约束
项目必须使用 `pnpm tauri dev` 命令启动以启用 Tauri API；在纯浏览器环境（如 `pnpm dev`）下 Tauri 功能不可用，需有明确提示。

### 3.2 Vite 开发服务器配置
- 开发服务器端口: `1420`（固定，不可冲突）
- 别名: `@` → `./src`
- 环境变量前缀: `VITE_` 和 `TAURI_`
- 其他: `clearScreen: false`（保留 Rust 错误输出）

### 3.3 项目构建脚本
- `npm run dev`: 启动 Vite 开发服务器
- `npm run build`: 先执行 `tsc` 类型检查，再执行 `vite build` 打包
- `npm run preview`: 启动打包后预览服务
- `npm run tauri`: 调用 Tauri CLI（如 `tauri dev`/`tauri build`）

### 3.4 SP 模板存储与渲染配置
Agent SP（System Prompt）模板以 Markdown 格式存放于 `src-tauri/resources/agents/` 目录，共 6 个文件，内容包含角色定义、可用工具列表、输出格式规范，并通过 `tera` 引擎注入项目名、用户偏好等运行时上下文。

---

## 四、开发规范

### 4.1 Tauri invoke 调用实践规范
所有 Tauri API 调用必须通过延迟获取 `invoke` 的方式实现（如 `getInvoke()` 辅助函数），禁止顶层静态导入，以规避 HMR 热更新或加载时序导致的 `invoke` 未定义问题。

### 4.2 MCP 风格工具注册规范
Agent 工具注册遵循 MCP（Model Context Protocol）风格，定义 `ToolDefinition`（含 name/description/parameters_schema）和 `ToolRegistry`（支持 handler 注册与动态调度），当前注册 8 个框架级工具（如 `task_decompose`、`memory_read` 等），handler 均为 Mock 实现。

---

## 五、重要决策

### 5.1 Tauri+React 项目启动阶段优先级决策
**场景**：多模块复杂桌面应用（Tauri+React）启动阶段的优先级排序

**决策内容**：
1. 首要任务：完成端到端可运行的脚手架（含前后端骨架、核心依赖、基础目录结构、编译验证）
2. 后续顺序：无限画布 → Agent 运行时 → API Gateway（遵循从 UI 呈现层到逻辑层再到服务层的自底向上构建逻辑）

### 5.2 项目管理 UI 入口决策
**场景**：为支持多项目协作，在 UI 中新增项目管理入口时需确定位置与交互形式

**决策内容**：
1. 主入口置于左侧导航栏（图标📂），与画布、设置并列
2. 项目模板选择集成在新建弹窗内，提供空白画布、短剧工作流、图文创作三类预设

### 5.3 MiniMax 风格节点交互与类型精简决策
**场景**：UI 界面改造中确定节点交互模式与类型范围

**决策内容**：
1. 节点交互模式：默认紧凑显示（图标+标题），双击展开为大卡片编辑态，点击画布空白处自动收起
2. 节点类型范围：严格限定为 5 种基础类型——文本、表格、图片、视频、音频，移除所有业务定制节点（如音乐、人物、分镜等）

### 5.4 内容平台能力对齐 MiniMax Hub 的节点扩展决策
**场景**：内容创作平台需与行业标杆（如 MiniMax Hub）能力对齐时，确定新增节点类型

**决策内容**：
1. 新增「音乐」节点，覆盖 BGM/人声歌曲/翻唱等专业音乐生成能力，区别于基础语音配音
2. 新增「人物」节点，构建角色一致性系统（character sheet pipeline），支持性别/年龄段/外貌特征等结构化描述

### 5.5 AI 密钥存储决策
**场景**：AI 供应商配置中权衡安全性与性能时确定密钥存储方式

**决策内容**：
1. 不对 API Key 进行本地加密（如 AES-256-GCM）
2. 采用明文存储以避免加解密开销导致的性能下降
3. 通过选用可信中转站供应商（而非直连原始 AI 服务商）来兼顾安全与效率

---

## 六、常见问题经验

### 6.1 TS 5.0+ 中禁用 baseUrl 配置项
TypeScript 5.0+ 版本已弃用 tsconfig.json 中的 "baseUrl" 选项，保留会导致 tsc 编译报错。

### 6.2 Tauri invoke 需动态获取避免 undefined
Tauri 应用中，顶层静态 `import { invoke }` 在 HMR 热更新或加载时序不稳时可能返回 undefined，必须改用动态获取方式：先检测 `window.__TAURI_INTERNALS__` 是否存在，再动态 `import('@tauri-apps/api').then(({ invoke }) => ...)` 并缓存。

### 6.3 TS 6.x 不支持 ignoreDeprecations 配置
TypeScript 6.x 中 `ignoreDeprecations` 编译选项已被移除，不再接受任何值；若需兼容旧版弃用警告，唯一合法值为 "5.0"，但实际应直接删除该配置项。

---

## 七、学习技能

### 7.1 Tauri API 安全调用技能
**输入**：Tauri 项目中需调用原生 API（如 invoke），需兼容开发环境与打包环境

**步骤**：
1. 创建 `getInvoke()` 辅助函数，首先检查 `window.__TAURI_INTERNALS__` 是否定义
2. 若定义，动态 `import('@tauri-apps/api')` 并解构 invoke，缓存结果
3. 若未定义（浏览器环境），抛出带上下文的中文错误提示
4. 所有 API 调用统一通过 `getInvoke()` 获取 invoke 实例

**注意事项**：
- 禁止在模块顶层静态 import { invoke }，必须封装为按需调用函数
- 缓存 invoke 实例避免重复 import 开销
- 错误提示需包含环境判断依据以便快速定位

### 7.2 前端编辑器撤销重做健壮实现技能
**输入**：编辑器中 undo/redo 快捷键失效，无法撤销/重做节点增删、连线、内容编辑、拖拽等操作

**步骤**：
1. 确保 history 初始化时包含一个空快照（避免 historyIndex <= 0 时直接 return）
2. 在 undo/redo 执行期间设置 `_isUndoRedo = true` 防护标志，阻止其触发的状态变更再次入栈
3. 在所有可撤销操作（如 updateNodeData、节点拖拽结束、连线增删）后调用 `pushHistory()`
4. loadCanvas 后重置 history 和 historyIndex，确保新画布从干净状态开始

**注意事项**：
- 防护标志必须在状态变更前设置、变更后清除，否则会漏保护或误拦截
- 拖拽结束事件和内容更新回调必须显式调用 pushHistory，不能依赖其他钩子

---

## 八、任务总结

### 8.1 画布 UI 全面改造
成功交付 UI 改造：节点支持双击展开/单击收起，5 类节点（文本/表格/图片/视频/音频）统一暗色双模式卡片；画布启用中键拖拽；右键菜单重写支持上传/添加节点/撤销重做/粘贴；连线改为灰色 Bezier 曲线并实现 hover 渐显 Handle；移除左下 Controls、左侧 Toolbar、NodeEditPanel 及非核心节点类型；保留 MiniMap、项目管理等基础功能。

### 8.2 画布节点 Delete/Backspace 快捷删除
成功实现安全、可撤销的节点快捷删除：全局监听 Delete/Backspace，精准拦截编辑态输入，支持多选删除，并与现有 Undo/Redo 系统无缝集成。

### 8.3 画布右键菜单清理与短剧工作流模板
成功交付：①右键菜单精简为纯内容创作节点（文本/图像/视频/音频/分镜/任务）；②工具栏新增🎬短剧按钮，点击即生成含 3 分镜→3 图像→3 视频+配音→合成的预连预填流水线；③tsconfig.json 移除 TS6+ 不兼容字段，确保编译零错误。

### 8.4 修复 Tauri 调用失败无错误提示问题
在 ProjectList.tsx 新建项目弹窗中新增 error 状态渲染区块（红色背景+文字提示），使 Tauri 调用失败时错误信息可显式展示，提升调试效率与用户体验。

### 8.5 项目管理功能补全
成功交付完整项目管理能力：导航栏新增📂图标入口，支持项目切换、新建（含短剧/图文等结构化模板）、重命名/删除，且切换前自动保存画布状态，填补了创作工作流的关键缺失环节。

### 8.6 AI 设置页集成中转站供应商并移除密钥加密
成功实现中转站供应商集成：API Key 改为明文本地存储，消除加密性能开销；设置页新增完整中转站配置界面，支持任意 OpenAI Chat Completions 兼容中转服务（如 one-api），用户只需填写 base_url 和 API Key 即可启用。

---

## 九、核心文件参考

### 画布 UI 改造核心文件
- `/src/canvas/nodes/TextNode.tsx`
- `/src/canvas/nodes/TableNode.tsx`
- `/src/canvas/nodes/ImageNode.tsx`
- `/src/canvas/nodes/VideoNode.tsx`
- `/src/canvas/nodes/AudioNode.tsx`
- `/src/canvas/ContextMenu.tsx`

---

*文档生成时间：2026-05-19*
