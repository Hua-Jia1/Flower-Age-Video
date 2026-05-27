import { useEffect, useState } from 'react';
import { listSkills, createSkill, updateSkill, deleteSkill, SkillInfo } from '../lib/tauri';

export default function SkillList() {
  const [skills, setSkills] = useState<SkillInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [showForm, setShowForm] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [formName, setFormName] = useState('');
  const [formDesc, setFormDesc] = useState('');
  const [formTemplate, setFormTemplate] = useState('');

  const loadData = async () => {
    try {
      const data = await listSkills();
      setSkills(data);
    } catch (e) {
      console.error('Failed to load skills:', e);
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
      await createSkill(formName.trim(), formDesc.trim(), formTemplate.trim());
      setFormName('');
      setFormDesc('');
      setFormTemplate('');
      setShowForm(false);
      await loadData();
    } catch (e) {
      console.error('Failed to create skill:', e);
    }
  };

  const handleUpdate = async () => {
    if (!editingId || !formName.trim()) return;
    try {
      await updateSkill(editingId, formName.trim(), formDesc.trim(), formTemplate.trim());
      setEditingId(null);
      setFormName('');
      setFormDesc('');
      setFormTemplate('');
      await loadData();
    } catch (e) {
      console.error('Failed to update skill:', e);
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('确定删除这个 Skill？')) return;
    try {
      await deleteSkill(id);
      await loadData();
    } catch (e) {
      console.error('Failed to delete skill:', e);
    }
  };

  const startEdit = (skill: SkillInfo) => {
    setEditingId(skill.id);
    setFormName(skill.name);
    setFormDesc(skill.description);
    setFormTemplate(skill.prompt_template);
    setShowForm(false);
  };

  const cancelEdit = () => {
    setEditingId(null);
    setFormName('');
    setFormDesc('');
    setFormTemplate('');
  };

  if (loading) {
    return <div className="p-8 flex items-center justify-center"><span className="text-muted-foreground">加载中...</span></div>;
  }

  return (
    <div className="p-8 max-w-4xl mx-auto">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold">Skill 管理</h1>
          <p className="text-sm text-muted-foreground mt-1">Skill 是文本模板，主 Agent 根据 Skill 创建并控制子 Agent 执行任务</p>
        </div>
        <button
          onClick={() => { setShowForm(true); setEditingId(null); setFormName(''); setFormDesc(''); setFormTemplate(''); }}
          className="px-4 py-2 rounded-md bg-primary text-primary-foreground text-sm hover:bg-primary/90"
        >
          + 新建 Skill
        </button>
      </div>

      {/* 创建表单 */}
      {showForm && (
        <div className="border rounded-lg p-4 mb-4 bg-muted/20">
          <h3 className="font-medium mb-3">新建 Skill</h3>
          <div className="space-y-3">
            <div>
              <label className="text-xs text-muted-foreground block mb-1">名称</label>
              <input
                type="text"
                value={formName}
                onChange={e => setFormName(e.target.value)}
                placeholder="例如：图片生成"
                className="w-full px-3 py-2 rounded-md border bg-background text-sm"
              />
            </div>
            <div>
              <label className="text-xs text-muted-foreground block mb-1">描述</label>
              <input
                type="text"
                value={formDesc}
                onChange={e => setFormDesc(e.target.value)}
                placeholder="简短描述这个Skill的用途"
                className="w-full px-3 py-2 rounded-md border bg-background text-sm"
              />
            </div>
            <div>
              <label className="text-xs text-muted-foreground block mb-1">Prompt 模板</label>
              <textarea
                value={formTemplate}
                onChange={e => setFormTemplate(e.target.value)}
                placeholder={`定义当用户请求相关任务时，如何创建子Agent完成任务。例如：\n\n当用户要求生成图片时，创建一个子Agent，使用以下system prompt：\n"你是一个专业的图片生成专家，根据用户的描述生成高质量图片..."\n然后调用图片生成工具完成请求。`}
                rows={8}
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

      {/* Skill 列表 */}
      <div className="space-y-4">
        {skills.map(skill => (
          <div key={skill.id} className="border rounded-lg p-4">
            {editingId === skill.id ? (
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
                  <label className="text-xs text-muted-foreground block mb-1">Prompt 模板</label>
                  <textarea value={formTemplate} onChange={e => setFormTemplate(e.target.value)} rows={8} className="w-full px-3 py-2 rounded-md border bg-background text-sm font-mono" />
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
                    <h3 className="font-semibold">{skill.name}</h3>
                    <span className="text-xs px-2 py-0.5 rounded-full bg-purple-100 text-purple-700">Skill</span>
                  </div>
                  <div className="flex gap-2">
                    <button onClick={() => startEdit(skill)} className="text-xs px-3 py-1 rounded-md border hover:bg-muted">编辑</button>
                    <button onClick={() => handleDelete(skill.id)} className="text-xs px-3 py-1 rounded-md border text-red-500 hover:bg-red-50">删除</button>
                  </div>
                </div>
                <p className="text-sm text-muted-foreground mb-2">{skill.description || '暂无描述'}</p>
                {skill.prompt_template && (
                  <div>
                    <label className="text-xs text-muted-foreground">Prompt 模板：</label>
                    <pre className="mt-1 p-2 bg-muted/50 rounded text-xs font-mono whitespace-pre-wrap">{skill.prompt_template}</pre>
                  </div>
                )}
              </div>
            )}
          </div>
        ))}
        {skills.length === 0 && (
          <div className="text-center py-12 text-muted-foreground">
            <p>暂无 Skill，点击上方按钮创建一个</p>
          </div>
        )}
      </div>
    </div>
  );
}