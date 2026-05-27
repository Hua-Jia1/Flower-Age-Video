/**
 * Tauri 2.x invoke 安全封装
 *
 * 背景：在某些情况下（HMR、初始化时序、非 Tauri 浏览器环境），直接顶层
 * `import { invoke } from '@tauri-apps/api/core'` 可能拿到 undefined
 * 导致 "Cannot read properties of undefined (reading 'invoke')"。
 * 此处统一通过 `getInvoke()` 进行环境检测 + 动态导入并校验。
 */

type InvokeFn = <T>(cmd: string, args?: Record<string, unknown>) => Promise<T>;

let cachedInvoke: InvokeFn | null = null;

async function getInvoke(): Promise<InvokeFn> {
  if (cachedInvoke) return cachedInvoke;

  // Tauri 2.x 环境检测
  if (typeof window === 'undefined' || !('__TAURI_INTERNALS__' in window)) {
    throw new Error('当前不在 Tauri 环境中运行，请使用 pnpm tauri dev 启动应用');
  }

  const mod = await import('@tauri-apps/api/core');
  const invokeFn = (mod as { invoke?: InvokeFn }).invoke;
  if (typeof invokeFn !== 'function') {
    throw new Error('Tauri invoke API 加载失败：@tauri-apps/api/core 未导出 invoke');
  }

  cachedInvoke = invokeFn;
  return invokeFn;
}

/**
 * 调用 Tauri 后端命令的封装
 */
export async function tauriInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const invoke = await getInvoke();
  return invoke<T>(cmd, args);
}

// 项目类型
export interface Project {
  id: string;
  name: string;
  description: string;
  created_at: string;
  updated_at: string;
}

// 画布数据类型
export interface CanvasNodeDto {
  id: string;
  type?: string;
  position: { x: number; y: number };
  data: Record<string, unknown>;
  width?: number;
  height?: number;
}

export interface CanvasEdgeDto {
  id: string;
  source: string;
  target: string;
  type?: string;
}

export interface CanvasData {
  nodes: CanvasNodeDto[];
  edges: CanvasEdgeDto[];
}

// === 项目 API ===

export async function createProject(name: string, description: string): Promise<Project> {
  const invoke = await getInvoke();
  return invoke<Project>('create_project', { name, description });
}

export async function listProjects(): Promise<Project[]> {
  const invoke = await getInvoke();
  return invoke<Project[]>('list_projects');
}

export async function getProject(id: string): Promise<Project | null> {
  const invoke = await getInvoke();
  return invoke<Project | null>('get_project', { id });
}

export async function updateProject(id: string, name: string, description: string): Promise<void> {
  const invoke = await getInvoke();
  return invoke<void>('update_project', { id, name, description });
}

export async function deleteProject(id: string): Promise<void> {
  const invoke = await getInvoke();
  return invoke<void>('delete_project', { id });
}

// === 画布 API ===

export async function saveCanvas(projectId: string, nodes: CanvasNodeDto[], edges: CanvasEdgeDto[]): Promise<void> {
  const invoke = await getInvoke();
  return invoke<void>('save_canvas', { projectId, nodes, edges });
}

export async function loadCanvas(projectId: string): Promise<CanvasData> {
  const invoke = await getInvoke();
  return invoke<CanvasData>('load_canvas', { projectId });
}

// ---- Agent API ----

export interface ChatMessage {
  role: string;
  content: string;
}

export interface GeneratedNode {
  node_type: string;
  label: string;
  data: Record<string, unknown>;
}

export interface AgentResponse {
  session_id: string;
  status: string;
  progress: number;
  summary: string;
  /** 完整的 AI 回复内容（自然语言） */
  message: string;
  generated_nodes: GeneratedNode[];
}

export interface SessionStatus {
  session_id: string;
  agent_name: string;
  state: string;
  step_count: number;
  max_steps: number;
  created_at: string;
  updated_at: string;
}

export interface SessionSummary {
  session_id: string;
  project_id: string;
  user_input: string;
  status: string;
  created_at: string;
}

/// 启动 Vibe Agent 会话（实际调用 start_vibe_session）
export async function startVibeSession(
  projectId: string,
  userInput: string,
  chatHistory: ChatMessage[]
): Promise<AgentResponse> {
  const invoke = await getInvoke();
  return invoke<AgentResponse>('start_vibe_session', {
    projectId,
    userInput,
    chatHistory,
  });
}

export async function getSessionStatus(sessionId: string): Promise<SessionStatus> {
  const invoke = await getInvoke();
  return invoke<SessionStatus>('get_session_status', { sessionId });
}

export async function listSessions(projectId: string): Promise<SessionSummary[]> {
  const invoke = await getInvoke();
  return invoke<SessionSummary[]>('list_sessions', { projectId });
}

export async function cancelSession(sessionId: string): Promise<void> {
  const invoke = await getInvoke();
  return invoke<void>('cancel_session', { sessionId });
}

// ---- API Key Management ----

export interface ApiKeyInfo {
  id: string;
  provider: string;
  label: string;
  base_url: string;
  model: string;
  is_active: boolean;
  created_at: string;
}

export async function saveApiKey(
  provider: string,
  label: string,
  apiKey: string,
  baseUrl?: string,
  model?: string,
): Promise<void> {
  const invoke = await getInvoke();
  return invoke<void>('save_api_key', {
    provider,
    label,
    apiKey,
    baseUrl: baseUrl ?? '',
    model: model ?? '',
  });
}

export async function deleteApiKey(provider: string, label: string): Promise<void> {
  const invoke = await getInvoke();
  return invoke<void>('delete_api_key', { provider, label });
}

export async function listApiKeys(): Promise<ApiKeyInfo[]> {
  const invoke = await getInvoke();
  return invoke<ApiKeyInfo[]>('list_api_keys');
}

export async function testApiKey(
  provider: string,
  apiKey: string,
  baseUrl?: string,
  model?: string,
): Promise<boolean> {
  const invoke = await getInvoke();
  return invoke<boolean>('test_api_key', {
    provider,
    apiKey,
    baseUrl: baseUrl ?? '',
    model: model ?? '',
  });
}

// === Agent API ===

export interface AgentInfo {
  id: string;
  name: string;
  agent_type: string;
  system_prompt: string;
  description: string;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface SkillInfo {
  id: string;
  name: string;
  description: string;
  prompt_template: string;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export async function createAgent(
  name: string,
  agentType: 'main' | 'sub',
  systemPrompt: string,
  description: string,
): Promise<AgentInfo> {
  const invoke = await getInvoke();
  return invoke<AgentInfo>('create_agent', { name, agentType, systemPrompt, description });
}

export async function listAgents(): Promise<AgentInfo[]> {
  const invoke = await getInvoke();
  return invoke<AgentInfo[]>('list_agents');
}

export async function getAgent(id: string): Promise<AgentInfo | null> {
  const invoke = await getInvoke();
  return invoke<AgentInfo | null>('get_agent', { id });
}

export async function updateAgent(
  id: string,
  name: string,
  systemPrompt: string,
  description: string,
): Promise<void> {
  const invoke = await getInvoke();
  return invoke<void>('update_agent', { id, name, systemPrompt, description });
}

export async function deleteAgent(id: string): Promise<void> {
  const invoke = await getInvoke();
  return invoke<void>('delete_agent', { id });
}

// === Skill API ===

export async function createSkill(
  name: string,
  description: string,
  promptTemplate: string,
): Promise<SkillInfo> {
  const invoke = await getInvoke();
  return invoke<SkillInfo>('create_skill', { name, description, promptTemplate });
}

export async function listSkills(): Promise<SkillInfo[]> {
  const invoke = await getInvoke();
  return invoke<SkillInfo[]>('list_skills');
}

export async function getSkill(id: string): Promise<SkillInfo | null> {
  const invoke = await getInvoke();
  return invoke<SkillInfo | null>('get_skill', { id });
}

export async function updateSkill(
  id: string,
  name: string,
  description: string,
  promptTemplate: string,
): Promise<void> {
  const invoke = await getInvoke();
  return invoke<void>('update_skill', { id, name, description, promptTemplate });
}

export async function deleteSkill(id: string): Promise<void> {
  const invoke = await getInvoke();
  return invoke<void>('delete_skill', { id });
}

// === Agent-Skill 关联 API ===

export async function assignSkillToAgent(agentId: string, skillId: string): Promise<void> {
  const invoke = await getInvoke();
  return invoke<void>('assign_skill_to_agent', { agentId, skillId });
}

export async function removeSkillFromAgent(agentId: string, skillId: string): Promise<void> {
  const invoke = await getInvoke();
  return invoke<void>('remove_skill_from_agent', { agentId, skillId });
}

export async function getSkillsForAgent(agentId: string): Promise<SkillInfo[]> {
  const invoke = await getInvoke();
  return invoke<SkillInfo[]>('get_skills_for_agent', { agentId });
}
