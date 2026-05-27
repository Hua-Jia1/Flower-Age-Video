import { useEffect, useState } from 'react';
import { Settings as SettingsIcon, LayoutGrid, FolderOpen, Bot, Sparkles } from 'lucide-react';
import Canvas from './canvas/Canvas';
import ChatPanel from './chat/ChatPanel';
import SettingsPage from './settings/SettingsPage';
import ProjectList from './projects/ProjectList';
import AgentList from './agents/AgentList';
import SkillList from './skills/SkillList';
import { useAutoSave } from './hooks/useAutoSave';
import { useCanvasStore } from './canvas/store';

type View = 'canvas' | 'settings' | 'projects' | 'agents' | 'skills';

function App() {
  useAutoSave();

  const [view, setView] = useState<View>('canvas');

  const loadCanvas = useCanvasStore((s) => s.loadCanvas);
  const setCurrentProject = useCanvasStore((s) => s.setCurrentProject);

  useEffect(() => {
    // 启动时初始化项目
    async function initProject() {
      try {
        const { listProjects, createProject } = await import('./lib/tauri');
        let projects = await listProjects();

        if (projects.length === 0) {
          // 首次使用，自动创建默认项目
          const project = await createProject('默认项目', '我的第一个 Flower Age Video 项目');
          projects = [project];
        }

        // 加载第一个项目的画布
        const currentProject = projects[0];
        setCurrentProject(currentProject.id);
        await loadCanvas(currentProject.id);
      } catch (err) {
        console.error('Failed to initialize project:', err);
      }
    }

    initProject();
  }, []);

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-background text-foreground" onContextMenu={(e) => e.preventDefault()}>
      {/* 左侧导航栏 */}
      <nav className="flex flex-col items-center gap-2 w-12 border-r bg-muted/30 py-3">
        <button
          onClick={() => setView('projects')}
          title="项目"
          className={`p-2 rounded-md transition-colors ${
            view === 'projects'
              ? 'bg-primary text-primary-foreground'
              : 'text-muted-foreground hover:bg-muted hover:text-foreground'
          }`}
        >
          <FolderOpen size={18} />
        </button>
        <button
          onClick={() => setView('canvas')}
          title="画布"
          className={`p-2 rounded-md transition-colors ${
            view === 'canvas'
              ? 'bg-primary text-primary-foreground'
              : 'text-muted-foreground hover:bg-muted hover:text-foreground'
          }`}
        >
          <LayoutGrid size={18} />
        </button>
        <button
          onClick={() => setView('settings')}
          title="设置"
          className={`p-2 rounded-md transition-colors ${
            view === 'settings'
              ? 'bg-primary text-primary-foreground'
              : 'text-muted-foreground hover:bg-muted hover:text-foreground'
          }`}
        >
          <SettingsIcon size={18} />
        </button>
        <button
          onClick={() => setView('agents')}
          title="Agent"
          className={`p-2 rounded-md transition-colors ${
            view === 'agents'
              ? 'bg-primary text-primary-foreground'
              : 'text-muted-foreground hover:bg-muted hover:text-foreground'
          }`}
        >
          <Bot size={18} />
        </button>
        <button
          onClick={() => setView('skills')}
          title="Skill"
          className={`p-2 rounded-md transition-colors ${
            view === 'skills'
              ? 'bg-primary text-primary-foreground'
              : 'text-muted-foreground hover:bg-muted hover:text-foreground'
          }`}
        >
          <Sparkles size={18} />
        </button>
      </nav>

      {view === 'canvas' ? (
        <>
          <div className="flex-1 relative">
            <Canvas />
          </div>
          <ChatPanel />
        </>
      ) : view === 'projects' ? (
        <div className="flex-1 overflow-auto">
          <ProjectList />
        </div>
      ) : view === 'agents' ? (
        <div className="flex-1 overflow-auto">
          <AgentList />
        </div>
      ) : view === 'skills' ? (
        <div className="flex-1 overflow-auto">
          <SkillList />
        </div>
      ) : (
        <div className="flex-1 overflow-auto">
          <SettingsPage />
        </div>
      )}
    </div>
  );
}

export default App;
