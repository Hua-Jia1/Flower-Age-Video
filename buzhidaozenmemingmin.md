# Flower Age Video（花纪视频工作室）— 项目设计文档

> **设计日期**：2026-05-18  
> **项目代号**：Flower Age Video  
> **目标平台**：Windows（.exe 单文件安装）  
> **核心模式**：主 Agent 编排 + 子 Agent 执行  
> **参考基线**：多Agent（架构模式） + TapCanvas Pro（无限画布） + 借鉴画布（插件系统）

---

## 一、项目定位

**Flower Age Video** 是一款 **AI 驱动的无限画布创意工作站**，面向视频创作者、内容生产者和 AI 艺术家。它在本地 Windows 桌面上提供一个**无限画布**，用户通过主 Agent 描述创作意图，由多个专业子 Agent 自动完成图像生成、视频生成、语音合成、音乐生成和后期制作的全链路协作。

### 核心价值主张

| 维度 | 说明 |
|---|---|
| **零商业侵权** | 全部依赖采用 MIT / Apache-2.0 / BSD 许可证，无 GPL 传染风险 |
| **本地部署** | 单文件 .exe 安装，无需 Docker、无需 Python 环境、无需 Node.js 运行时 |
| **云端 AI** | 用户配置自己的 API Key，本地加密存储，无第三方代理 |
| **无限画布** | 所有资产（图片/视频/音频/文本/分镜）在画布上可视化管理 |
| **主 Agent 控子 Agent** | 1 个编排器拆解任务 → 5 个专业子 Agent 并行/串行执行 |

---

## 二、整体架构

```
┌──────────────────────────────────────────────────────────────┐
│  Tauri Desktop Shell (FlowerAgeVideo.exe)                     │  ← 桌面壳层
│  ├── Rust Backend (核心引擎)                                   │
│  │   ├── Agent Runtime        主/子 Agent 生命周期管理          │
│  │   ├── Tool Registry        MCP 风格工具注册表                │
│  │   ├── API Gateway          统一 API 代理 (localhost:7942)    │
│  │   ├── Plugin Engine        热加载插件系统                    │
│  │   ├── Key Vault            API Key 加密存储 (AES-256-GCM)   │
│  │   └── SQLite Store         项目/资产/Memory 持久化           │
│  ├── React WebView (前端)                                      │
│  │   ├── Infinite Canvas      @xyflow/react 无限画布            │
│  │   ├── Agent Chat           主 Agent 对话面板                  │
│  │   ├── Asset Browser        资产管理浏览器                     │
│  │   ├── Storyboard Editor    分镜编辑器                        │
│  │   └── Settings             模型/API/偏好设置                  │
│  └── Native Modules                                            │
│      ├── ffmpeg (内置)        媒体编解码/合并/转场               │
│      ├── Sharp (Rust)         高性能图像处理                     │
│      └── Whisper (可选)       本地语音转文字                     │
└──────────────────────────────────────────────────────────────┘
```

### 2.1 各层职责

| 层次 | 技术 | 作用 |
|---|---|---|
| **Tauri 壳** | Tauri 2.x (Rust) | 桌面窗口容器、系统托盘、自动更新、原生 IPC |
| **Agent Runtime** | Rust 异步运行时 | Agent 生命周期管理、工具调度、SP 注入、步数限制 |
| **API Gateway** | Rust (axum/hyper) | 本地 API 代理，路由到各 AI 供应商 API |
| **Plugin Engine** | Rust (动态加载) | 支持 `.wasm` 插件热加载，扩展新 AI 模型接入 |
| **Key Vault** | Rust (ring/aead) | API Key 本地加密存储，Windows DPAPI 加持 |
| **Infinite Canvas** | React + @xyflow/react | 节点式画布，拖拽/缩放/连线/分组 |
| **Agent Chat** | React + SSE | 实时 Agent 对话与进度反馈 |
| **SQLite** | tauri-plugin-sql | 项目数据、资产索引、Memory、会话历史 |
| **ffmpeg** | ffmpeg-sidecar (MIT) | 视频合并、转场、音频混音、格式转换 |

### 2.2 请求链路

```
用户输入意图 (Chat)
    │
    ▼
Agent Runtime (主 Agent: flowerage-orchestrator)
    │ 拆解任务 → 构建执行计划
    │
    ├──► image-agent    ──► API Gateway ──► 云端图像 API (Midjourney/DALL·E/Stability/etc.)
    ├──► video-agent    ──► API Gateway ──► 云端视频 API (Runway/Kling/Seedance/etc.)
    ├──► speech-agent   ──► API Gateway ──► 云端语音 API (MiniMax TTS/ElevenLabs/etc.)
    ├──► music-agent    ──► API Gateway ──► 云端音乐 API (Suno/Udio/etc.)
    └──► editing-agent  ──► ffmpeg 本地执行 ──► 合并渲染输出
    │
    ▼
Canvas Sync (所有生成物自动注册到画布节点)
```

---

## 三、技术栈（全部 MIT / Apache-2.0 许可证）

### 3.1 桌面框架

| 组件 | 技术 | 许可证 | 说明 |
|---|---|---|---|
| 桌面框架 | **Tauri 2.x** | MIT + Apache-2.0 | 比 Electron 小 ~90%，Rust 后端 |
| WebView | WebView2 (Windows 内置) | 系统组件 | Windows 10+ 预装，零额外依赖 |
| 打包工具 | Tauri Bundler | MIT | 生成 .msi / .exe 安装包 |
| 自动更新 | Tauri Updater | MIT | 增量更新，无需额外服务器 |

### 3.2 前端

| 组件 | 技术 | 许可证 | 说明 |
|---|---|---|---|
| UI 框架 | **React 18** | MIT | 成熟生态 |
| 类型系统 | TypeScript 5.x | Apache-2.0 | 全链路类型安全 |
| 构建工具 | **Vite** | MIT | 极速 HMR |
| 无限画布 | **@xyflow/react** (React Flow) | MIT | 行业标准无限画布 |
| 状态管理 | **Zustand** | MIT | 轻量、高性能 |
| UI 组件 | **shadcn/ui** + Radix | MIT | 无运行时依赖，复制即用 |
| 样式 | Tailwind CSS | MIT | 原子化 CSS |
| 动画 | Framer Motion | MIT | 声明式动画 |
| AI 对话 | @ai-sdk/react (Vercel AI SDK) | Apache-2.0 | 支持多 LLM 供应商 |

### 3.3 后端 (Rust)

| 组件 | Crate | 许可证 | 说明 |
|---|---|---|---|
| HTTP 框架 | **axum** | MIT | 高性能异步 HTTP |
| 异步运行时 | **tokio** | MIT | Rust 异步标准 |
| JSON | serde / serde_json | MIT + Apache-2.0 | 序列化标准 |
| HTTP 客户端 | reqwest | MIT + Apache-2.0 | 调用云端 API |
| 加密 | **ring** + aead | ISC (BSD-like) | AES-256-GCM 加密 API Key |
| SQLite | rusqlite | MIT | 嵌入式数据库 |
| 模板引擎 | tera | MIT | Agent SP 模板渲染 |
| SSE | tokio-stream | MIT | Server-Sent Events 进度推送 |
| 插件 | wasmtime | Apache-2.0 | WASM 插件运行时 |

### 3.4 原生模块

| 组件 | 技术 | 许可证 | 说明 |
|---|---|---|---|
| 媒体处理 | **ffmpeg-sidecar** | MIT | 自动下载管理 ffmpeg 二进制 |
| 图像处理 | **image-rs** | MIT | Rust 原生图像编解码 |
| 音频处理 | symphonia | MPL-2.0 (非传染) | Rust 原生音频解码 |
| 语音识别(可选) | whisper-rs | MIT | 本地语音转文字 |

---

## 四、Agent 体系（核心设计）

### 4.1 分层架构

```
┌─────────────────────────────────────────────┐
│  flowerage-orchestrator (主 Agent)           │
│  角色：总编排器                               │
│  职责：理解用户意图 → 拆解任务 → 派发子Agent   │
│       → 汇总结果 → 更新画布                    │
│  模式：primary                                │
│  步数预算：400                                │
│  工具集：任务拆解 / 画布读写 / Memory / 派发    │
└──────┬──────┬──────┬──────┬──────────────────┘
       │      │      │      │
       ▼      ▼      ▼      ▼
┌──────┐ ┌────┐ ┌────┐ ┌────┐ ┌────────┐
│image │ │vid │ │spe │ │mus │ │editing │
│Agent │ │Agent│ │Agent│ │Agent│ │Agent   │
│      │ │    │ │    │ │    │ │        │
│ 200  │ │200 │ │200 │ │200 │ │ 200    │ ← 步数预算
└──────┘ └────┘ └────┘ └────┘ └────────┘
```

### 4.2 Agent 详细定义

| Agent ID | 角色 | 核心能力 | 步数预算 | MCP 工具数 |
|---|---|---|---|---|
| `flowerage-orchestrator` | 总编排器 | 意图识别 → 任务拆解 → 派发 → 汇总 → 画布同步 | 400 | 8 |
| `image-agent` | 图像专家 | T2I / I2I / 角色一致性 / 多模型路由 / 批量生成 | 200 | 6 |
| `video-agent` | 视频专家 | T2V / I2V / 多模态视频 / 运动控制 / 数字人 | 200 | 6 |
| `speech-agent` | 语音专家 | TTS / 声音克隆 / 多角色对话 / 情绪控制 | 200 | 5 |
| `music-agent` | 音乐专家 | BGM / 人声歌曲 / 翻唱 / 歌词创作 | 200 | 5 |
| `editing-agent` | 后期专家 | 片段合并 / 转场 / 唇形同步 / BGM 混音 / MV 流水线 | 200 | 8 |

### 4.3 主 Agent 编排流程

```
Stage 1: 意图理解
  ├── 解析用户输入
  ├── 识别任务类型 (单模态 / 多模态 / 全流程)
  └── 输出：任务拆解计划

Stage 2: 并行派发
  ├── 无依赖的 Agent → 并行执行（如 image + music 同时生成）
  ├── 有依赖的 Agent → 串行执行（如 image 输出 → video 输入）
  └── 输出：各子 Agent 执行结果

Stage 3: 汇总与画布同步
  ├── 校验所有子 Agent 输出
  ├── 调用 Canvas API 注册节点
  ├── 建立 sourceNodeId 资产关系链
  └── 输出：用户可读的总结 + 画布更新
```

### 4.4 Agent 系统提示词设计

每个 Agent 的 SP 包含：

```
┌─────────────────────────────────┐
│ 1. 角色定义 + 能力边界            │
│ 2. 工具使用 SOP (标准操作流程)     │
│ 3. 降级矩阵 (失败兜底策略)         │
│ 4. 输出格式规范                   │
│ 5. 安全红线 (不可覆盖的约束)       │
│ 6. 步数预算提醒                   │
└─────────────────────────────────┘
```

- 主 Agent SP：~500 行，核心是 Stage 决策表 + 派发规则
- 子 Agent SP：~200 行/个，核心是模型路由 + 降级矩阵
- **总计**：~1500 行 SP，模板化管理在 `resources/agents/` 下

---

## 五、无限画布设计

### 5.1 画布核心

基于 **@xyflow/react (React Flow)** 的无限画布，支持：

- **无限缩放**：0.1x ~ 5x，鼠标滚轮
- **拖拽布局**：自由拖拽节点
- **连线关系**：节点间有向边，表达资产依赖
- **分组节点**：Group Node 收纳相关资产
- **小地图**：MiniMap 全局导航
- **对齐吸附**：拖拽自动吸附对齐

### 5.2 节点类型

| 节点类型 | 图标 | 功能 | 数据承载 |
|---|---|---|---|
| **ProjectNode** | 📁 | 项目根节点 | 项目名称/描述/创建时间 |
| **TextNode** | 📝 | 文本/脚本/创意 | Markdown 内容 |
| **ImageNode** | 🖼️ | 图像资产 | 缩略图 + 提示词 + 模型 + 尺寸 |
| **VideoNode** | 🎬 | 视频资产 | 预览帧 + 提示词 + 时长 + 分辨率 |
| **AudioNode** | 🎵 | 音频资产 | 波形 + 时长 + 情绪标签 |
| **StoryboardNode** | 🎞️ | 分镜节点 | 镜头号 + 描述 + 时长 + 转场 |
| **TaskNode** | ⚙️ | Agent 任务节点 | 任务状态 + 进度 + Agent 日志 |
| **GroupNode** | 📦 | 分组容器 | 子节点集合 |

### 5.3 画布状态管理 (Zustand)

```typescript
interface CanvasState {
  // 节点与边
  nodes: Node[];
  edges: Edge[];
  
  // 操作
  addNode: (node: Node) => void;
  removeNode: (id: string) => void;
  connectNodes: (source: string, target: string) => void;
  
  // 画布视口
  viewport: { x: number; y: number; zoom: number };
  
  // 项目上下文
  currentProjectId: string;
  
  // Agent 集成
  agentTaskMap: Map<string, string>; // taskId → nodeId
  syncAgentResult: (taskId: string, result: AgentResult) => void;
}
```

---

## 六、云端 AI API 集成

### 6.1 API Gateway 设计

用户自己在本地配置各供应商的 API Key，**不经过任何第三方服务器**：

```
用户设置 → Key Vault (AES-256-GCM 加密存储)
    │
    ▼
Agent Runtime → API Gateway (localhost:7942)
    │
    ├──► OpenAI API        (api.openai.com)
    ├──► Anthropic API     (api.anthropic.com)
    ├──► Stability API     (api.stability.ai)
    ├──► Midjourney API    (通过第三方代理)
    ├──► Runway API        (api.runwayml.com)
    ├──► Kling API         (api.klingai.com)
    ├──► MiniMax API       (api.minimax.chat)
    ├──► ElevenLabs API    (api.elevenlabs.io)
    ├──► Suno API          (api.suno.ai)
    └──► ...更多供应商
```

### 6.2 支持的 AI 供应商（首批）

#### 大语言模型（LLM — 驱动 Agent 推理）

| 供应商 | 模型 | 用途 |
|---|---|---|
| OpenAI | GPT-4o / GPT-4.1 | 主力 Agent 推理 |
| Anthropic | Claude 4 Sonnet / Opus | 复杂创意推理 |
| DeepSeek | DeepSeek-V3 / V4 | 高性价比推理 |
| 通义千问 | Qwen-Max | 中文优化 |
| MiniMax | MiniMax-Text-01 | 备用推理 |

#### 图像生成

| 供应商 | 模型 | 用途 |
|---|---|---|
| OpenAI | DALL·E 3 | 通用图像 |
| Stability AI | SD3 / SDXL | 可控图像 |
| Midjourney | (第三方代理) | 艺术风格 |
| 即梦/豆包 | Seedream | 中文文字渲染 |
| Recraft | V3 | 矢量/设计 |

#### 视频生成

| 供应商 | 模型 | 用途 |
|---|---|---|
| Runway | Gen-3 / Gen-4 | 高质量 T2V |
| 可灵 | Kling 2.x | 中文短视频 |
| 即梦 | Seedance 2.0 | 多模态视频 |
| MiniMax | Video-01 | 通用视频 |
| Luma | Dream Machine | 氛围视频 |

#### 语音合成

| 供应商 | 模型 | 用途 |
|---|---|---|
| MiniMax | speech-2.8-hd | 中文多情绪 TTS |
| ElevenLabs | Multilingual v2 | 多语种 + 声音克隆 |
| OpenAI | TTS-1 / TTS-1-hd | 通用 TTS |

#### 音乐生成

| 供应商 | 模型 | 用途 |
|---|---|---|
| Suno | V4 | 带人声歌曲 |
| Udio | V2 | 高品质 BGM |
| MiniMax | music-2.6 | 器乐/翻唱 |

### 6.3 API Key 安全存储

```
┌─────────────────────────────────────────────┐
│  Key Vault 安全设计                           │
│                                               │
│  1. API Key → AES-256-GCM 加密               │
│  2. 加密密钥 → Windows DPAPI 保护             │
│  3. 密文存储 → SQLite (本地)                  │
│  4. 解密仅在 API 调用时，内存中短暂停留        │
│  5. 从不写入日志、不网络传输、不持久化明文     │
└─────────────────────────────────────────────┘
```

---

## 七、项目结构

```
FlowerAgeVideo/
├── src-tauri/                    # Tauri Rust 后端
│   ├── src/
│   │   ├── main.rs               # 入口
│   │   ├── agent/                 # Agent 运行时
│   │   │   ├── runtime.rs         # Agent 生命周期管理
│   │   │   ├── orchestrator.rs    # 主 Agent 编排器
│   │   │   ├── sub_agent.rs       # 子 Agent 执行器
│   │   │   ├── tool_registry.rs   # MCP 风格工具注册
│   │   │   └── sp_loader.rs       # 系统提示词加载
│   │   ├── gateway/               # API 网关
│   │   │   ├── mod.rs
│   │   │   ├── router.rs          # 模型路由
│   │   │   ├── providers/         # 各供应商适配器
│   │   │   │   ├── openai.rs
│   │   │   │   ├── anthropic.rs
│   │   │   │   ├── stability.rs
│   │   │   │   ├── minimax.rs
│   │   │   │   ├── runway.rs
│   │   │   │   └── ...
│   │   │   └── rate_limiter.rs    # 速率限制
│   │   ├── vault/                 # Key Vault
│   │   │   ├── mod.rs
│   │   │   ├── encrypt.rs         # AES-256-GCM
│   │   │   └── dpapi.rs           # Windows DPAPI
│   │   ├── plugin/                # 插件引擎
│   │   │   ├── engine.rs          # WASM 加载器
│   │   │   └── manifest.rs        # 插件清单
│   │   ├── storage/               # 存储
│   │   │   ├── db.rs              # SQLite 操作
│   │   │   ├── models.rs          # 数据模型
│   │   │   └── migrations/        # 数据库迁移
│   │   ├── canvas/                # 画布后端
│   │   │   ├── sync.rs            # 画布同步协议
│   │   │   └── node_store.rs      # 节点持久化
│   │   └── media/                 # 媒体处理
│   │       ├── ffmpeg.rs          # ffmpeg 命令封装
│   │       └── thumbnail.rs       # 缩略图生成
│   ├── resources/
│   │   ├── agents/                # Agent 系统提示词
│   │   │   ├── flowerage-orchestrator.md
│   │   │   ├── image-agent.md
│   │   │   ├── video-agent.md
│   │   │   ├── speech-agent.md
│   │   │   ├── music-agent.md
│   │   │   └── editing-agent.md
│   │   ├── contracts/             # 合约/约束协议
│   │   │   ├── anti-loop.md
│   │   │   ├── canvas-discipline.md
│   │   │   ├── model-routing.md
│   │   │   ├── output-format.md
│   │   │   └── memory-discipline.md
│   │   └── knowledge/             # 知识库
│   │       ├── color-theory.md
│   │       ├── composition.md
│   │       ├── lighting.md
│   │       ├── character-design.md
│   │       └── ...
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── src/                           # React 前端
│   ├── main.tsx                   # 入口
│   ├── App.tsx                    # 根组件
│   ├── canvas/                    # 画布模块
│   │   ├── Canvas.tsx             # 主画布组件
│   │   ├── store.ts               # Zustand 状态
│   │   ├── hooks/                 # 画布 Hooks
│   │   │   ├── useCanvasNodes.ts
│   │   │   ├── useCanvasEdges.ts
│   │   │   └── useCanvasSync.ts
│   │   ├── nodes/                 # 节点类型
│   │   │   ├── ProjectNode.tsx
│   │   │   ├── TextNode.tsx
│   │   │   ├── ImageNode.tsx
│   │   │   ├── VideoNode.tsx
│   │   │   ├── AudioNode.tsx
│   │   │   ├── StoryboardNode.tsx
│   │   │   ├── TaskNode.tsx
│   │   │   └── GroupNode.tsx
│   │   └── toolbar/               # 画布工具栏
│   │       ├── Toolbar.tsx
│   │       └── MiniMap.tsx
│   ├── chat/                      # Agent 对话
│   │   ├── ChatPanel.tsx          # 对话面板
│   │   ├── MessageBubble.tsx      # 消息气泡
│   │   ├── AgentThinking.tsx      # Agent 思考过程展示
│   │   └── ProgressBar.tsx        # 任务进度条
│   ├── assets/                    # 资产管理
│   │   ├── AssetBrowser.tsx       # 资产浏览器
│   │   └── AssetCard.tsx          # 资产卡片
│   ├── storyboard/                # 分镜编辑器
│   │   ├── StoryboardEditor.tsx
│   │   ├── ShotCard.tsx
│   │   └── Timeline.tsx
│   ├── settings/                  # 设置
│   │   ├── SettingsPage.tsx
│   │   ├── ApiKeyManager.tsx      # API Key 管理
│   │   ├── ModelSelector.tsx      # 模型选择
│   │   └── PreferencePanel.tsx    # 偏好设置
│   ├── projects/                  # 项目管理
│   │   ├── ProjectList.tsx        # 项目列表
│   │   └── ProjectCreate.tsx      # 创建项目
│   ├── components/ui/             # shadcn/ui 组件
│   └── lib/                       # 工具函数
│       ├── tauri.ts               # Tauri IPC 封装
│       ├── api.ts                  # API 调用
│       └── utils.ts
│
├── package.json
├── tsconfig.json
├── vite.config.ts
├── tailwind.config.ts
├── index.html
└── README.md
```

---

## 八、本地安装方案（零 Docker）

### 8.1 安装包形式

| 格式 | 大小 | 说明 |
|---|---|---|
| **FlowerAgeVideo-Setup.exe** | ~60MB | NSIS/MSI 安装向导，一键安装 |
| **FlowerAgeVideo.msi** | ~58MB | 企业静默部署 |

### 8.2 安装流程

```
1. 下载 FlowerAgeVideo-Setup.exe (~60MB)
2. 双击运行安装向导
3. 选择安装目录 (默认: %LOCALAPPDATA%\FlowerAgeVideo)
4. 安装程序自动：
   ├── 解压 Tauri 二进制 (~8MB)
   ├── 解压前端资源 (~15MB)
   ├── 检测/下载 WebView2 (如果没有，自动安装 ~1MB)
   ├── 下载 ffmpeg (~35MB，首次启动时)
   └── 创建桌面快捷方式
5. 完成！双击桌面图标启动
```

### 8.3 运行时依赖

| 依赖 | 来源 | 安装方式 |
|---|---|---|
| WebView2 | Windows 10+ 内置 | 无需安装（Windows 10 1809+ 预装） |
| ffmpeg | 自动下载 | 首次启动时从 GitHub Releases 下载 |
| VC++ Runtime | 系统内置 | Windows 自带 |

> **零用户负担**：用户不需要安装 Python、Node.js、Docker、任何运行时。双击 exe 即可使用。

---

## 九、存储设计

### 9.1 本地数据库 (SQLite)

```
flowerage.db
├── projects            # 项目元信息
├── nodes               # 画布节点 (JSON)
├── edges               # 画布连线
├── assets              # 资产索引 (路径/缩略图/元数据)
├── sessions            # Agent 会话历史
├── memory              # Memory 持久化 (用户偏好/角色锚点)
├── api_keys            # 加密的 API Key
├── settings            # 用户设置
└── plugin_registry     # 已安装插件
```

### 9.2 文件系统布局

```
%LOCALAPPDATA%\FlowerAgeVideo\
├── flowerage.db               # SQLite 数据库
├── settings.json              # 非敏感设置
├── agents/                    # 用户自定义 Agent SP
├── plugins/                   # 用户安装的 WASM 插件
├── assets/                    # 生成资产
│   ├── images/
│   ├── videos/
│   ├── audio/
│   └── thumbnails/
├── projects/                  # 项目导出
├── logs/                      # 运行日志
└── ffmpeg/                    # ffmpeg 二进制
```

---

## 十、关键设计亮点

### 10.1 从 多Agent 借鉴的设计

| 设计 | 来源 | 实现方式 |
|---|---|---|
| **主 Agent + 子 Agent 分层** | 多Agent | `orchestrator` → 5 个专业 Agent |
| **确定性派发流程** | 多Agent Stage 1→3 | 三步流水线：意图→派发→汇总 |
| **步数预算控制** | 多Agent 200-400 | 每个 Agent 设置 max_turns |
| **Canvas 节点系统** | 多Agent | 全部资产通过 canvas_write_node 注册 |
| **Memory 持久化** | 多Agent | 跨会话风格偏好 + 角色锚点 |
| **知识库 Top-5 截断** | 多Agent | 扫描→打分→Top-5→一次性读取 |
| **Medium Lock** | 多Agent | Prompt 末尾嵌入媒介锁定约束 |
| **合约约束系统** | 多Agent 12 合约 | 精简为 6 个核心合约 |
| **流校验规则** | 多Agent editing-agent | ffmpeg 后校验 Δ ≤ 0.08s |
| **降级矩阵** | 多Agent | 每个 Agent 内置失败兜底链 |

### 10.2 从 TapCanvas Pro 借鉴的设计

| 设计 | 来源 | 实现方式 |
|---|---|---|
| **无限画布** | TapCanvas Pro @xyflow/react | React Flow + Zustand |
| **节点类型系统** | TapCanvas Pro | 8 种节点类型 |
| **Skills 系统** | TapCanvas Pro skills/ | Agent SP 模板化 |
| **Agents Bridge** | TapCanvas Pro | Tauri IPC 桥接 |
| **多 Agent 协作** | TapCanvas Pro spawn_agent | Orchestrator 并行/串行派发 |

### 10.3 从借鉴画布的设计

| 设计 | 来源 | 实现方式 |
|---|---|---|
| **插件热加载** | 借鉴画布 Python importlib | WASM 动态加载 |
| **SSE 进度推送** | 借鉴画布 | tokio-stream SSE |
| **多供应商并行** | 借鉴画布 | 统一 API Gateway 路由 |
| **智能模型检测** | 借鉴画布 detect-models | 自动探测 API 端点能力 |

### 10.4 独创设计

| 设计 | 说明 |
|---|---|
| **Tauri 轻量壳** | 抛弃 Electron，exe 从 150MB 缩减到 60MB |
| **API Key 本地加密** | 无第三方云代理，AES-256-GCM + DPAPI |
| **WASM 插件** | 安全沙箱插件，比 Python Cython .pyd 更安全 |
| **零环境依赖** | 不需要 Docker/Python/Node.js，双击即用 |

---

## 十一、许可证合规性审查

### 全部依赖许可证

| 组件 | 许可证 | 商业使用 | 修改分发 | 传染性 |
|---|---|---|---|---|
| Tauri | MIT + Apache-2.0 | ✅ | ✅ | ❌ |
| React | MIT | ✅ | ✅ | ❌ |
| @xyflow/react | MIT | ✅ | ✅ | ❌ |
| Zustand | MIT | ✅ | ✅ | ❌ |
| shadcn/ui | MIT | ✅ | ✅ | ❌ |
| Tailwind CSS | MIT | ✅ | ✅ | ❌ |
| Framer Motion | MIT | ✅ | ✅ | ❌ |
| Vercel AI SDK | Apache-2.0 | ✅ | ✅ | ❌ |
| axum | MIT | ✅ | ✅ | ❌ |
| tokio | MIT | ✅ | ✅ | ❌ |
| reqwest | MIT + Apache-2.0 | ✅ | ✅ | ❌ |
| ring | ISC (BSD-like) | ✅ | ✅ | ❌ |
| rusqlite | MIT | ✅ | ✅ | ❌ |
| wasmtime | Apache-2.0 | ✅ | ✅ | ❌ |
| ffmpeg-sidecar | MIT | ✅ | ✅ | ❌ |
| image-rs | MIT | ✅ | ✅ | ❌ |
| symphonia | MPL-2.0 | ✅ | ✅ | ❌ (非传染) |

**结论：100% 无 GPL 传染性许可证，可安全商用。**

### 与参考项目的差异（避免侵权）

| 维度 | 多Agent | Flower Age Video | 侵权风险 |
|---|---|---|---|
| 桌面框架 | Electron (MIT) | Tauri (MIT+Apache-2.0) | ❌ 无 |
| Agent 引擎 | OpenCode (闭源) | 自研 Rust Agent Runtime | ❌ 无 |
| MCP 实现 | @modelcontextprotocol/sdk | 自研 MCP 风格工具注册 | ❌ 无（MCP 是开放协议） |
| SP 内容 | MiniMax 专有 | 完全自编写 | ❌ 无 |
| 品牌 | 多Agent | Flower Age Video | ❌ 无 |
| 云端网关 | MiniMax Cloud Gateway | 直连各 AI 供应商 API | ❌ 无 |

---

## 十二、开发路线图

### Phase 1：核心框架（4 周）

- [ ] Tauri 项目脚手架搭建
- [ ] React 前端框架搭建（Vite + React + TypeScript）
- [ ] 无限画布基础（React Flow 集成）
- [ ] 画布节点类型实现（Project / Text / Image / Video / Task）
- [ ] Zustand 状态管理
- [ ] SQLite 数据库初始化
- [ ] Tauri IPC 前后端通信

### Phase 2：Agent 运行时（4 周）

- [ ] Agent Runtime 核心（生命周期/工具调度/SP 注入）
- [ ] 主 Agent (orchestrator) SP 编写
- [ ] 子 Agent SP 编写（image / video / speech / music / editing）
- [ ] MCP 风格工具注册表
- [ ] Agent 对话前端（ChatPanel + SSE 进度）
- [ ] 步数限制 + 降级矩阵实现

### Phase 3：API 网关（3 周）

- [ ] API Gateway 基础框架（axum）
- [ ] OpenAI / Anthropic 适配器
- [ ] 图像 API 适配器（DALL·E / Stability / Midjourney）
- [ ] 视频 API 适配器（Runway / Kling / Seedance）
- [ ] 语音 API 适配器（MiniMax TTS / ElevenLabs）
- [ ] 音乐 API 适配器（Suno / Udio）
- [ ] Key Vault 加密存储

### Phase 4：画布增强（2 周）

- [ ] Canvas 节点自动注册（Agent 结果 → 画布节点）
- [ ] 分镜编辑器（Storyboard Editor）
- [ ] 资产浏览器（Asset Browser）
- [ ] Memory 系统（偏好/角色锚点持久化）
- [ ] 知识库 + Top-5 截断协议

### Phase 5：后期制作（2 周）

- [ ] ffmpeg 集成（ffmpeg-sidecar）
- [ ] editing-agent 完整流程
- [ ] 视频合并/转场/BGM 混音
- [ ] 流校验（Δ ≤ 0.08s）
- [ ] MP4/MOV/AVI 多格式导出

### Phase 6：打包与发布（2 周）

- [ ] Tauri 打包配置（Windows .msi/.exe）
- [ ] 自动更新配置
- [ ] 安装向导设计
- [ ] 首次启动引导流程
- [ ] 端到端测试

**总计：~17 周**

---

## 十三、与 多Agent 的对比

| 维度 | 多Agent | Flower Age Video |
|---|---|---|
| **桌面框架** | Electron (~150MB) | Tauri (~60MB) |
| **Agent 引擎** | OpenCode (闭源) | 自研 Rust Agent Runtime (开源) |
| **后端语言** | Node.js (Gateway + MCP) | Rust (全后端) |
| **API 模式** | Cloud Gateway 代理（用户无 Key） | 用户自有 API Key 直连 |
| **本地模型** | 不支持 | 可选 WASM 插件扩展 |
| **插件系统** | OpenCode Plugin API | WASM 沙箱插件 |
| **无限画布** | Canvas 节点（有限画布） | React Flow 真正无限画布 |
| **分镜编辑** | 无 | 内置 Storyboard Editor |
| **安装依赖** | 需要系统环境 | 零依赖，双击即用 |
| **许可证** | 闭源商业 | 开源 MIT（核心）+ 自研 |
| **价格** | 按量付费（Cloud Gateway） | 免费软件 + 自带 API Key |

---

## 十四、总结

Flower Age Video 是一个**站在巨人肩膀上**的设计方案：

- 从 **多Agent** 借鉴了成熟的主Agent+子Agent编排模式、确定性派发流程、Canvas节点系统、Memory持久化、知识库截断协议、Medium Lock等核心设计理念
- 从 **TapCanvas Pro** 借鉴了无限画布架构、React Flow集成方案、Skills系统和Agents Bridge模式
- 从 **借鉴画布** 借鉴了插件热加载、多供应商并行、SSE进度推送的设计思路

但在**技术选型**上做了彻底的差异化：
- 用 **Tauri** 替代 Electron（体积减少60%，安全增强）
- 用 **Rust** 替代 Node.js/Python（性能提升，单二进制部署）
- 用 **用户自有 API Key** 替代 Cloud Gateway（零中间商，隐私优先）
- 用 **WASM 插件** 替代 Python Cython（更安全的沙箱）

所有依赖 **100% MIT/Apache-2.0/BSD 许可证**，无任何商业侵权风险。

---

> **设计者注**：本项目名"Flower Age Video"（花纪视频工作室）灵感来源于创作如花般绽放的时代。让 AI 成为你的创作伙伴，而不是替代者。
