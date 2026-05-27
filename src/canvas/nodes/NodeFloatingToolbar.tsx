import { memo } from 'react';
import { useReactFlow } from '@xyflow/react';
import { useCanvasStore } from '../store';

interface NodeFloatingToolbarProps {
  /** 当前节点 ID，用于复制等操作 */
  nodeId: string;
  /** 工具栏首个按钮的文字（如 "生成音频" / "生成视频" / "生成文本"） */
  primaryLabel: string;
  /** 首按钮的点击回调，预留给后续 Agent 系统接入 */
  onPrimary?: () => void;
  /** "添加到对话" 按钮回调，预留 */
  onAddToChat?: () => void;
  /** 全屏按钮回调，预留 */
  onFullscreen?: () => void;
}

/**
 * 节点上方悬浮工具栏，仅在 hover 时显示。
 * 视觉对齐 MiniMax Hub：圆角胶囊 + 暗色玻璃背景 + 分隔线。
 */
function NodeFloatingToolbar({
  nodeId,
  primaryLabel,
  onPrimary,
  onAddToChat,
  onFullscreen,
}: NodeFloatingToolbarProps) {
  const { fitView } = useReactFlow();
  const stop = (e: React.SyntheticEvent) => e.stopPropagation();

  // 默认回调：主按钮 — 闪烁节点边框（视觉反馈）
  const handlePrimary = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (onPrimary) {
      onPrimary();
    } else {
      // 默认行为：闪烁节点边框表示操作已触发
      const el = document.querySelector(`[data-id="${nodeId}"]`) as HTMLElement | null;
      if (el) {
        el.style.transition = 'box-shadow 0.15s ease';
        el.style.boxShadow = '0 0 0 2px #3b82f6';
        setTimeout(() => { el.style.boxShadow = ''; }, 500);
      }
    }
  };

  // 默认回调：添加到对话 — 复制节点数据到剪贴板
  const handleAddToChat = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (onAddToChat) {
      onAddToChat();
    } else {
      const node = useCanvasStore.getState().nodes.find(n => n.id === nodeId);
      if (node) {
        navigator.clipboard.writeText(JSON.stringify(node.data, null, 2)).catch(() => {});
        const el = document.querySelector(`[data-id="${nodeId}"]`) as HTMLElement | null;
        if (el) {
          el.style.transition = 'box-shadow 0.15s ease';
          el.style.boxShadow = '0 0 0 2px #22c55e';
          setTimeout(() => { el.style.boxShadow = ''; }, 500);
        }
      }
    }
  };

  // 默认回调：复制
  const handleCopy = (e: React.MouseEvent) => {
    e.stopPropagation();
    useCanvasStore.getState().copyNode(nodeId);
    const el = document.querySelector(`[data-id="${nodeId}"]`) as HTMLElement | null;
    if (el) {
      el.style.transition = 'box-shadow 0.15s ease';
      el.style.boxShadow = '0 0 0 2px #a855f7';
      setTimeout(() => { el.style.boxShadow = ''; }, 500);
    }
  };

  // 默认回调：全屏 — 聚焦到当前节点
  const handleFullscreen = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (onFullscreen) {
      onFullscreen();
    } else {
      const node = useCanvasStore.getState().nodes.find(n => n.id === nodeId);
      if (node) {
        fitView({ nodes: [{ id: nodeId }], duration: 400, padding: 0.3 });
      }
    }
  };

  return (
    <div
      className="nodrag absolute -top-10 left-1/2 -translate-x-1/2 flex items-center gap-0.5 bg-zinc-900/95 border border-zinc-700/60 rounded-lg px-1 py-1 shadow-lg shadow-black/40 backdrop-blur-sm opacity-0 group-hover:opacity-100 transition-opacity duration-200 z-10"
      onMouseDown={stop}
      onClick={stop}
    >
      <button
        type="button"
        className="px-2.5 py-1 text-xs text-zinc-300 hover:bg-zinc-700/60 rounded whitespace-nowrap transition-colors"
        onClick={handlePrimary}
        onMouseDown={stop}
      >
        {primaryLabel}
      </button>
      <div className="w-px h-4 bg-zinc-700/60 mx-0.5" />
      <button
        type="button"
        className="p-1.5 text-zinc-400 hover:text-zinc-200 hover:bg-zinc-700/60 rounded transition-colors"
        title="复制节点数据到剪贴板"
        onClick={handleAddToChat}
        onMouseDown={stop}
      >
        <svg
          className="w-4 h-4"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
        >
          <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
          <line x1="12" y1="8" x2="12" y2="14" />
          <line x1="9" y1="11" x2="15" y2="11" />
        </svg>
      </button>
      <button
        type="button"
        className="p-1.5 text-zinc-400 hover:text-zinc-200 hover:bg-zinc-700/60 rounded transition-colors"
        title="复制节点"
        onClick={handleCopy}
        onMouseDown={stop}
      >
        <svg
          className="w-4 h-4"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
        >
          <rect x="9" y="9" width="13" height="13" rx="2" />
          <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
        </svg>
      </button>
      <button
        type="button"
        className="p-1.5 text-zinc-400 hover:text-zinc-200 hover:bg-zinc-700/60 rounded transition-colors"
        title="聚焦节点"
        onClick={handleFullscreen}
        onMouseDown={stop}
      >
        <svg
          className="w-4 h-4"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
        >
          <polyline points="15 3 21 3 21 9" />
          <polyline points="9 21 3 21 3 15" />
          <line x1="21" y1="3" x2="14" y2="10" />
          <line x1="3" y1="21" x2="10" y2="14" />
        </svg>
      </button>
    </div>
  );
}

export default memo(NodeFloatingToolbar);