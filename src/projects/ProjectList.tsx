import { useEffect, useState, useRef } from 'react';
import {
  FolderOpen,
  Plus,
  Trash2,
  Pencil,
  Check,
  X,
  MoreVertical,
  Sparkles,
  LayoutGrid,
} from 'lucide-react';
import {
  listProjects,
  createProject,
  updateProject,
  deleteProject,
  Project,
} from '../lib/tauri';
import { useCanvasStore } from '../canvas/store';

type TemplateKey = 'blank';

const TEMPLATES: { key: TemplateKey; title: string; desc: string; icon: typeof LayoutGrid }[] = [
  { key: 'blank', title: '空白画布', desc: '从零开始的纯净起点', icon: LayoutGrid },
];

function formatDate(iso: string): string {
  try {
    const d = new Date(iso);
    return d.toLocaleString('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    });
  } catch {
    return iso;
  }
}

export default function ProjectList() {
  const [projects, setProjects] = useState<Project[]>([]);
  const [loading, setLoading] = useState(true);
  const [menuOpenId, setMenuOpenId] = useState<string | null>(null);
  const [renamingId, setRenamingId] = useState<string | null>(null);
  const [renameValue, setRenameValue] = useState('');
  const [deleteTarget, setDeleteTarget] = useState<Project | null>(null);
  const [createOpen, setCreateOpen] = useState(false);
  const [creating, setCreating] = useState(false);
  const [newName, setNewName] = useState('');
  const [newDesc, setNewDesc] = useState('');
  const [template, setTemplate] = useState<TemplateKey>('blank');
  const [error, setError] = useState<string | null>(null);

  const menuRef = useRef<HTMLDivElement | null>(null);

  const currentProjectId = useCanvasStore((s) => s.currentProjectId);
  const setCurrentProject = useCanvasStore((s) => s.setCurrentProject);
  const loadCanvas = useCanvasStore((s) => s.loadCanvas);
  const saveCanvas = useCanvasStore((s) => s.saveCanvas);
  const setNodes = useCanvasStore((s) => s.setNodes);
  const setEdges = useCanvasStore((s) => s.setEdges);

  useEffect(() => {
    refresh();
  }, []);

  useEffect(() => {
    function onClick(e: MouseEvent) {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        setMenuOpenId(null);
      }
    }
    document.addEventListener('mousedown', onClick);
    return () => document.removeEventListener('mousedown', onClick);
  }, []);

  async function refresh() {
    setLoading(true);
    try {
      const list = await listProjects();
      // 按更新时间倒序
      list.sort((a, b) => (a.updated_at < b.updated_at ? 1 : -1));
      setProjects(list);
    } catch (err) {
      console.error('Failed to load projects:', err);
    } finally {
      setLoading(false);
    }
  }

  async function switchTo(id: string) {
    if (id === currentProjectId) return;
    // 先保存当前画布
    if (currentProjectId) {
      try {
        await saveCanvas();
      } catch (err) {
        console.error('Failed to save before switch:', err);
      }
    }
    setCurrentProject(id);
    await loadCanvas(id);
  }

  function startRename(p: Project) {
    setRenamingId(p.id);
    setRenameValue(p.name);
    setMenuOpenId(null);
  }

  async function commitRename(p: Project) {
    const next = renameValue.trim();
    setRenamingId(null);
    if (!next || next === p.name) return;
    try {
      await updateProject(p.id, next, p.description);
      await refresh();
    } catch (err) {
      console.error('Failed to rename:', err);
    }
  }

  async function confirmDelete() {
    if (!deleteTarget) return;
    const target = deleteTarget;
    setDeleteTarget(null);
    try {
      await deleteProject(target.id);
      const remaining = projects.filter((p) => p.id !== target.id);

      if (target.id === currentProjectId) {
        if (remaining.length > 0) {
          const next = remaining[0];
          setCurrentProject(next.id);
          await loadCanvas(next.id);
        } else {
          // 没有项目了，创建一个默认项目
          const fresh = await createProject('默认项目', '我的第一个 Flower Age Video 项目');
          setCurrentProject(fresh.id);
          setNodes([]);
          setEdges([]);
          await saveCanvas();
        }
      }
      await refresh();
    } catch (err) {
      console.error('Failed to delete:', err);
    }
  }

  function openCreate() {
    setNewName('');
    setNewDesc('');
    setTemplate('blank');
    setCreateOpen(true);
  }

  async function submitCreate() {
    const name = newName.trim();
    if (!name || creating) return;
    setCreating(true);
    setError(null);
    try {
      // 切换前保存当前（失败不阻塞创建）
      if (currentProjectId) {
        try {
          await saveCanvas();
        } catch (err) {
          console.error('Pre-create save failed:', err);
        }
      }

      const created = await createProject(name, newDesc.trim());

      // 切换到新项目并清空画布
      setCurrentProject(created.id);
      setNodes([]);
      setEdges([]);

      // 写入画布数据（含模板）
      try {
        await saveCanvas();
      } catch (err) {
        console.error('Post-create save failed:', err);
      }
      await refresh();
      setCreateOpen(false);
    } catch (err: any) {
      const msg = err?.message || err?.toString() || '未知错误';
      console.error('Failed to create project:', msg);
      setError(`创建失败: ${msg}`);
    } finally {
      setCreating(false);
    }
  }

  return (
    <div className="relative h-full w-full overflow-auto bg-background">
      {/* 装饰性背景：网格 + 角落辉光 */}
      <div
        aria-hidden
        className="pointer-events-none absolute inset-0 opacity-[0.18]"
        style={{
          backgroundImage:
            'linear-gradient(to right, currentColor 1px, transparent 1px), linear-gradient(to bottom, currentColor 1px, transparent 1px)',
          backgroundSize: '32px 32px',
          color: 'hsl(var(--muted-foreground))',
          maskImage:
            'radial-gradient(ellipse at top left, black 0%, transparent 70%)',
          WebkitMaskImage:
            'radial-gradient(ellipse at top left, black 0%, transparent 70%)',
        }}
      />
      <div
        aria-hidden
        className="pointer-events-none absolute -top-32 -right-32 h-96 w-96 rounded-full blur-3xl opacity-30"
        style={{ background: 'radial-gradient(circle, #6366f1 0%, transparent 70%)' }}
      />

      <div className="relative max-w-5xl mx-auto px-8 py-10">
        {/* 标题区 */}
        <header className="flex items-end justify-between mb-10 pb-6 border-b">
          <div>
            <div className="flex items-center gap-2 text-xs uppercase tracking-[0.3em] text-muted-foreground mb-3">
              <span className="inline-block h-px w-8 bg-current" />
              Workspace
            </div>
            <h1 className="text-4xl font-bold tracking-tight">项目管理</h1>
            <p className="mt-2 text-sm text-muted-foreground">
              管理画布与工作流 · 共 <span className="font-semibold text-foreground">{projects.length}</span> 个项目
            </p>
          </div>
          <button
            onClick={openCreate}
            className="group inline-flex items-center gap-2 px-4 py-2.5 rounded-md bg-primary text-primary-foreground text-sm font-medium shadow-sm hover:bg-primary/90 transition-all hover:shadow-md hover:-translate-y-0.5"
          >
            <Plus size={16} className="transition-transform group-hover:rotate-90" />
            新建项目
          </button>
        </header>

        {/* 内容区 */}
        {loading ? (
          <div className="flex items-center justify-center py-24">
            <span className="text-sm text-muted-foreground">加载中...</span>
          </div>
        ) : projects.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-24 text-center">
            <div className="h-16 w-16 rounded-full bg-muted/50 flex items-center justify-center mb-4">
              <FolderOpen size={28} className="text-muted-foreground" />
            </div>
            <h3 className="text-lg font-semibold mb-1">还没有项目</h3>
            <p className="text-sm text-muted-foreground mb-4">从一个模板开始你的第一个创作</p>
            <button
              onClick={openCreate}
              className="inline-flex items-center gap-2 px-4 py-2 rounded-md border text-sm hover:bg-muted transition-colors"
            >
              <Plus size={14} /> 新建项目
            </button>
          </div>
        ) : (
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
            {projects.map((p) => {
              const isCurrent = p.id === currentProjectId;
              const isRenaming = renamingId === p.id;
              return (
                <article
                  key={p.id}
                  onClick={() => !isRenaming && switchTo(p.id)}
                  className={[
                    'group relative cursor-pointer rounded-lg border p-5 transition-all',
                    'hover:border-primary/60 hover:shadow-md hover:-translate-y-0.5',
                    isCurrent
                      ? 'border-primary bg-primary/5 shadow-[0_0_0_1px_hsl(var(--primary))]'
                      : 'border-border bg-card',
                  ].join(' ')}
                >
                  {/* 当前活跃指示条 */}
                  {isCurrent && (
                    <span
                      aria-hidden
                      className="absolute left-0 top-4 bottom-4 w-1 rounded-r bg-primary"
                    />
                  )}

                  <div className="flex items-start justify-between gap-2 mb-3">
                    <div className="flex items-center gap-2 min-w-0 flex-1">
                      <FolderOpen
                        size={16}
                        className={isCurrent ? 'text-primary' : 'text-muted-foreground'}
                      />
                      {isRenaming ? (
                        <input
                          autoFocus
                          value={renameValue}
                          onChange={(e) => setRenameValue(e.target.value)}
                          onClick={(e) => e.stopPropagation()}
                          onKeyDown={(e) => {
                            if (e.key === 'Enter') commitRename(p);
                            if (e.key === 'Escape') setRenamingId(null);
                          }}
                          onBlur={() => commitRename(p)}
                          className="flex-1 min-w-0 px-2 py-1 text-sm border rounded bg-background"
                        />
                      ) : (
                        <h3 className="font-semibold text-sm truncate">{p.name}</h3>
                      )}
                      {isCurrent && !isRenaming && (
                        <span className="shrink-0 inline-flex items-center gap-1 px-1.5 py-0.5 text-[10px] font-medium uppercase tracking-wider rounded bg-primary/10 text-primary">
                          <Sparkles size={10} /> 当前
                        </span>
                      )}
                    </div>

                    {/* 操作菜单 */}
                    {!isRenaming && (
                      <div className="relative" ref={menuOpenId === p.id ? menuRef : null}>
                        <button
                          onClick={(e) => {
                            e.stopPropagation();
                            setMenuOpenId(menuOpenId === p.id ? null : p.id);
                          }}
                          className="p-1 rounded text-muted-foreground hover:bg-muted hover:text-foreground opacity-0 group-hover:opacity-100 transition-opacity data-[open=true]:opacity-100"
                          data-open={menuOpenId === p.id}
                        >
                          <MoreVertical size={14} />
                        </button>
                        {menuOpenId === p.id && (
                          <div
                            onClick={(e) => e.stopPropagation()}
                            className="absolute right-0 top-7 z-10 min-w-[140px] rounded-md border bg-popover text-popover-foreground shadow-lg py-1 text-sm"
                          >
                            <button
                              onClick={() => startRename(p)}
                              className="w-full flex items-center gap-2 px-3 py-1.5 hover:bg-muted text-left"
                            >
                              <Pencil size={13} /> 重命名
                            </button>
                            <button
                              onClick={() => {
                                setMenuOpenId(null);
                                setDeleteTarget(p);
                              }}
                              className="w-full flex items-center gap-2 px-3 py-1.5 hover:bg-red-50 dark:hover:bg-red-900/20 text-red-600 text-left"
                            >
                              <Trash2 size={13} /> 删除
                            </button>
                          </div>
                        )}
                      </div>
                    )}
                  </div>

                  <p className="text-xs text-muted-foreground line-clamp-2 min-h-[2rem] mb-4">
                    {p.description || <span className="italic opacity-60">没有描述</span>}
                  </p>

                  <div className="flex items-center justify-between text-[11px] text-muted-foreground pt-3 border-t border-dashed">
                    <span>更新于</span>
                    <span className="font-mono">{formatDate(p.updated_at)}</span>
                  </div>
                </article>
              );
            })}
          </div>
        )}
      </div>

      {/* 新建项目对话框 */}
      {createOpen && (
        <Modal onClose={() => !creating && setCreateOpen(false)}>
          <div className="px-6 py-5 border-b flex items-center justify-between">
            <div>
              <h2 className="text-lg font-semibold">新建项目</h2>
              <p className="text-xs text-muted-foreground mt-0.5">为你的下一个创作搭建画布</p>
            </div>
            <button
              onClick={() => !creating && setCreateOpen(false)}
              className="p-1 rounded text-muted-foreground hover:bg-muted hover:text-foreground"
            >
              <X size={16} />
            </button>
          </div>

          <div className="px-6 py-5 space-y-5">
            <div>
              <label className="block text-xs font-medium uppercase tracking-wider text-muted-foreground mb-1.5">
                项目名称
              </label>
              <input
                autoFocus
                value={newName}
                onChange={(e) => setNewName(e.target.value)}
                placeholder="例如：春日短剧 #01"
                className="w-full px-3 py-2 rounded-md border bg-background text-sm focus:outline-none focus:ring-2 focus:ring-primary/40"
              />
            </div>

            <div>
              <label className="block text-xs font-medium uppercase tracking-wider text-muted-foreground mb-1.5">
                描述 <span className="text-muted-foreground/60 normal-case tracking-normal">（可选）</span>
              </label>
              <textarea
                value={newDesc}
                onChange={(e) => setNewDesc(e.target.value)}
                rows={2}
                placeholder="一句话描述项目..."
                className="w-full px-3 py-2 rounded-md border bg-background text-sm resize-none focus:outline-none focus:ring-2 focus:ring-primary/40"
              />
            </div>

            <div>
              <label className="block text-xs font-medium uppercase tracking-wider text-muted-foreground mb-2">
                选择模板
              </label>
              <div className="grid grid-cols-3 gap-2">
                {TEMPLATES.map((t) => {
                  const Icon = t.icon;
                  const active = template === t.key;
                  return (
                    <button
                      key={t.key}
                      type="button"
                      onClick={() => setTemplate(t.key)}
                      className={[
                        'relative text-left rounded-md border p-3 transition-all',
                        active
                          ? 'border-primary bg-primary/5 shadow-[0_0_0_1px_hsl(var(--primary))]'
                          : 'border-border hover:border-primary/40 hover:bg-muted/40',
                      ].join(' ')}
                    >
                      {active && (
                        <Check
                          size={12}
                          className="absolute top-2 right-2 text-primary"
                          strokeWidth={3}
                        />
                      )}
                      <Icon
                        size={18}
                        className={active ? 'text-primary mb-2' : 'text-muted-foreground mb-2'}
                      />
                      <div className="text-sm font-medium">{t.title}</div>
                      <div className="text-[10px] text-muted-foreground mt-0.5 leading-snug">
                        {t.desc}
                      </div>
                    </button>
                  );
                })}
              </div>
            </div>
          </div>

          {error && (
            <div className="px-6 py-2 bg-red-50 dark:bg-red-900/20 border-t border-red-200 dark:border-red-800">
              <p className="text-xs text-red-600 dark:text-red-400">{error}</p>
            </div>
          )}
          <div className="px-6 py-4 border-t bg-muted/20 flex justify-end gap-2">
            <button
              onClick={() => setCreateOpen(false)}
              disabled={creating}
              className="px-4 py-2 text-sm rounded-md border hover:bg-muted transition-colors disabled:opacity-50"
            >
              取消
            </button>
            <button
              onClick={submitCreate}
              disabled={!newName.trim() || creating}
              className="px-4 py-2 text-sm rounded-md bg-primary text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-50 inline-flex items-center gap-1.5"
            >
              {creating ? '创建中...' : (<><Plus size={14} /> 创建</>)}
            </button>
          </div>
        </Modal>
      )}

      {/* 删除确认对话框 */}
      {deleteTarget && (
        <Modal onClose={() => setDeleteTarget(null)}>
          <div className="px-6 py-5">
            <div className="flex items-start gap-3">
              <div className="shrink-0 h-10 w-10 rounded-full bg-red-100 dark:bg-red-900/30 flex items-center justify-center">
                <Trash2 size={18} className="text-red-600" />
              </div>
              <div className="flex-1 min-w-0">
                <h2 className="text-base font-semibold">删除项目？</h2>
                <p className="text-sm text-muted-foreground mt-1">
                  即将永久删除 <span className="font-medium text-foreground">「{deleteTarget.name}」</span>
                  ，此操作不可撤销。
                </p>
              </div>
            </div>
          </div>
          <div className="px-6 py-4 border-t bg-muted/20 flex justify-end gap-2">
            <button
              onClick={() => setDeleteTarget(null)}
              className="px-4 py-2 text-sm rounded-md border hover:bg-muted transition-colors"
            >
              取消
            </button>
            <button
              onClick={confirmDelete}
              className="px-4 py-2 text-sm rounded-md bg-red-600 text-white hover:bg-red-700 transition-colors inline-flex items-center gap-1.5"
            >
              <Trash2 size={14} /> 确认删除
            </button>
          </div>
        </Modal>
      )}
    </div>
  );
}

function Modal({ children, onClose }: { children: React.ReactNode; onClose: () => void }) {
  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm animate-in fade-in"
      onClick={onClose}
    >
      <div
        onClick={(e) => e.stopPropagation()}
        className="w-full max-w-lg bg-background border rounded-lg shadow-2xl overflow-hidden"
      >
        {children}
      </div>
    </div>
  );
}
