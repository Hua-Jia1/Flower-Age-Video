import { useEffect, useState } from 'react';
import { listAgents, createAgent, updateAgent, deleteAgent, listSkills, assignSkillToAgent, removeSkillFromAgent, getSkillsForAgent, AgentInfo, SkillInfo } from '../lib/tauri';

export default function AgentList() {
  const [agents, setAgents] = useState<AgentInfo[]>([]);
  const [skills, setSkills] = useState<SkillInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [showForm, setShowForm] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [formName, setFormName] = useState('');
  const [formPrompt, setFormPrompt] = useState('');
  const [formDesc, setFormDesc] = useState('');
  const [agentSkills, setAgentSkills] = useState<Record<string, SkillInfo[]>>({});

  const loadData = async () => {
    try {
      const [agentsData, skillsData] = await Promise.all([listAgents(), listSkills()]);
      setAgents(agentsData);
      setSkills(skillsData);

      // Load skills for each agent
      const skillsMap: Record<string, SkillInfo[]> = {};
      for (const agent of agentsData) {
        if (agent.agent_type === 'sub') {
          skillsMap[agent.id] = await getSkillsForAgent(agent.id);
        }
      }
      setAgentSkills(skillsMap);
    } catch (e) {
      console.error('Failed to load agents:', e);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadData();
  }, []);

  const handleCreate = async () => {
    if (!formName.trim()) return;
    try {
      await createAgent(formName.trim(), 'sub', formPrompt.trim(), formDesc.trim());
      setFormName('');
      setFormPrompt('');
      setFormDesc('');
      setShowForm(false);
      await loadData();
    } catch (e) {
      console.error('Failed to create agent:', e);
    }
  };

  const handleUpdate = async () => {
    if (!editingId || !formName.trim()) return;
    try {
      await updateAgent(editingId, formName.trim(), formPrompt.trim(), formDesc.trim());
      setEditingId(null);
      setFormName('');
      setFormPrompt('');
      setFormDesc('');
      await loadData();
    } catch (e) {
      console.error('Failed to update agent:', e);
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('确定删除这个 Agent？')) return;
    try {
      await deleteAgent(id);
      await loadData();
    } catch (e) {
      console.error('Failed to delete agent:', e);
    }
  };

  const startEdit = (agent: AgentInfo) => {
    setEditingId(agent.id);
    setFormName(agent.name);
    setFormPrompt(agent.system_prompt);
    setFormDesc(agent.description);
    setShowForm(false);
  };

  const cancelEdit = () => {
    setEditingId(null);
    setFormName('');
    setFormPrompt('');
    setFormDesc('');
  };

  const toggleSkill = async (agentId: string, skillId: string, hasSkill: boolean) => {
    try {
      if (hasSkill) {
        await removeSkillFromAgent(agentId, skillId);
      } else {
        await assignSkillToAgent(agentId, skillId);
      }
      await loadData();
    } catch (e) {
      console.error('Failed to toggle skill:', e);
    }
  };

  if (loading) {
    return <div className="p-8 flex items-center justify-center"><span className="text-muted-foreground">加载中...</span></div>;
  }

  return (
    <div className="p-8 max-w-4xl mx-auto">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold">Agent 管理</h1>
          <p className="text-sm text-muted-foreground mt-1">管理主 Agent 和子 Agent，系统 prompt 定义子 Agent 的行为</p>
        </div>
        <button
          onClick={() => { setShowForm(true); setEditingId(null); setFormName(''); setFormPrompt(''); setFormDesc(''); }}
          className="px-4 py-2 rounded-md bg-primary text-primary-foreground text-sm hover:bg-primary/90"
        >
          + 新建子 Agent
        </button>
      </div>

      {/* 创建表单 */}
      {showForm && (
        <div className="border rounded-lg p-4 mb-4 bg-muted/20">
          <h3 className="font-medium mb-3">新建子 Agent</h3>
          <div className="space-y-3">
            <div>
              <label className="text-xs text-muted-foreground block mb-1">名称</label>
              <input
                type="text"
                value={formName}
                onChange={e => setFormName(e.target.value)}
                placeholder="例如：图片生成Agent"
                className="w-full px-3 py-2 rounded-md border bg-background text-sm"
              />
            </div>
            <div>
              <label className="text-xs text-muted-foreground block mb-1">描述</label>
              <input
                type="text"
                value={formDesc}
                onChange={e => setFormDesc(e.target.value)}
                placeholder="简短描述这个Agent的职责"
                className="w-full px-3 py-2 rounded-md border bg-background text-sm"
              />
            </div>
            <div>
              <label className="text-xs text-muted-foreground block mb-1">System Prompt</label>
              <textarea
                value={formPrompt}
                onChange={e => setFormPrompt(e.target.value)}
                placeholder="定义这个Agent如何执行任务，例如：你是一个图片生成专家，当用户要求生成图片时，调用图片生成工具..."
                rows={6}
                className="w-full px-3 py-2 rounded-md border bg-background text-sm font-mono"
              />
            </div>
            <div className="flex gap-2">
              <button onClick={handleCreate} className="px-4 py-2 rounded-md bg-primary text-primary-foreground text-sm">保存</button>
              <button onClick={() => setShowForm(false)} className="px-4 py-2 rounded-md border text-sm">取消</button>
            </div>
          </div>
        </div>
      )}

      {/* Agent 列表 */}
      <div className="space-y-4">
        {agents.map(agent => (
          <div key={agent.id} className="border rounded-lg p-4">
            {editingId === agent.id ? (
              /* 编辑模式 */
              <div className="space-y-3">
                <div>
                  <label className="text-xs text-muted-foreground block mb-1">名称</label>
                  <input type="text" value={formName} onChange={e => setFormName(e.target.value)} className="w-full px-3 py-2 rounded-md border bg-background text-sm" />
                </div>
                <div>
                  <label className="text-xs text-muted-foreground block mb-1">描述</label>
                  <input type="text" value={formDesc} onChange={e => setFormDesc(e.target.value)} className="w-full px-3 py-2 rounded-md border bg-background text-sm" />
                </div>
                <div>
                  <label className="text-xs text-muted-foreground block mb-1">System Prompt</label>
                  <textarea value={formPrompt} onChange={e => setFormPrompt(e.target.value)} rows={6} className="w-full px-3 py-2 rounded-md border bg-background text-sm font-mono" />
                </div>
                <div className="flex gap-2">
                  <button onClick={handleUpdate} className="px-4 py-2 rounded-md bg-primary text-primary-foreground text-sm">保存</button>
                  <button onClick={cancelEdit} className="px-4 py-2 rounded-md border text-sm">取消</button>
                </div>
              </div>
            ) : (
              /* 显示模式 */
              <div>
                <div className="flex items-center justify-between mb-2">
                  <div className="flex items-center gap-2">
                    <h3 className="font-semibold">{agent.name}</h3>
                    <span className={`text-xs px-2 py-0.5 rounded-full ${agent.agent_type === 'main' ? 'bg-blue-100 text-blue-700' : 'bg-green-100 text-green-700'}`}>
                      {agent.agent_type === 'main' ? '主Agent' : '子Agent'}
                    </span>
                  </div>
                  <div className="flex gap-2">
                    {agent.agent_type === 'sub' && (
                      <button onClick={() => startEdit(agent)} className="text-xs px-3 py-1 rounded-md border hover:bg-muted">编辑</button>
                    )}
                    {agent.agent_type === 'sub' && (
                      <button onClick={() => handleDelete(agent.id)} className="text-xs px-3 py-1 rounded-md border text-red-500 hover:bg-red-50">删除</button>
                    )}
                  </div>
                </div>
                <p className="text-sm text-muted-foreground mb-2">{agent.description || '暂无描述'}</p>
                {agent.system_prompt && (
                  <div className="mb-3">
                    <label className="text-xs text-muted-foreground">System Prompt：</label>
                    <pre className="mt-1 p-2 bg-muted/50 rounded text-xs font-mono whitespace-pre-wrap">{agent.system_prompt}</pre>
                  </div>
                )}
                {agent.agent_type === 'sub' && (
                  <div>
                    <label className="text-xs text-muted-foreground mb-1 block">关联的 Skills：</label>
                    <div className="flex flex-wrap gap-2 mt-1">
                      {skills.map(skill => {
                        const hasSkill = agentSkills[agent.id]?.some(s => s.id === skill.id);
                        return (
                          <button
                            key={skill.id}
                            onClick={() => toggleSkill(agent.id, skill.id, hasSkill)}
                            className={`text-xs px-2 py-1 rounded-full border transition-colors ${hasSkill ? 'bg-primary text-primary-foreground' : 'bg-muted/50 text-muted-foreground'}`}
                          >
                            {skill.name}
                          </button>
                        );
                      })}
                      {skills.length === 0 && <span className="text-xs text-muted-foreground">暂无 Skills，请先创建 Skill</span>}
                    </div>
                  </div>
                )}
              </div>
            )}
          </div>
        ))}
        {agents.length === 0 && (
          <div className="text-center py-12 text-muted-foreground">
            <p>暂无 Agent，点击上方按钮创建子 Agent</p>
          </div>
        )}
      </div>
    </div>
  );
}