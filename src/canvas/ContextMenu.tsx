import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from 'react';
import {
  ChevronRight,
  Copy,
  Edit3,
  Image as ImageIcon,
  Maximize2,
  Plus,
  Redo2,
  Table2,
  Trash2,
  Type as TypeIcon,
  Undo2,
  Unlink,
  Upload,
  Video as VideoIcon,
  Volume2,
  ClipboardPaste,
} from 'lucide-react';
import { useReactFlow } from '@xyflow/react';
import { useCanvasStore, type CanvasNodeType } from './store';

// 添加节点子菜单项
const NODE_ITEMS: { type: CanvasNodeType; icon: ReactNode; label: string }[] = [
  { type: 'text', icon: <TypeIcon size={15} />, label: '文本' },
  { type: 'table', icon: <Table2 size={15} />, label: '表格' },
  { type: 'image', icon: <ImageIcon size={15} />, label: '图片' },
  { type: 'video', icon: <VideoIcon size={15} />, label: '视频' },
  { type: 'audio', icon: <Volume2 size={15} />, label: '音频' },
];

// 估算菜单尺寸用于边界检测
const MENU_WIDTH = 224;
const MENU_HEIGHT_PANE = 260;
const MENU_HEIGHT_NODE = 220;
const SUBMENU_WIDTH = 200;
const SUBMENU_HEIGHT = 220;

export default function ContextMenu() {
  const {
    contextMenu,
    hideContextMenu,
    addNodeAtPosition,
    updateNodeData,
    removeNode,
    copyNode,
    disconnectNode,
    expandNode,
    pasteNode,
    undo,
    redo,
  } = useCanvasStore();

  const { fitView } = useReactFlow();

  const { visible, x, y, flowX, flowY, nodeId } = contextMenu;

  const fileInputRef = useRef<HTMLInputElement>(null);
  const [submenuOpen, setSubmenuOpen] = useState(false);
  const closeTimerRef = useRef<number | null>(null);

  // 关闭菜单时一并重置子菜单状态
  useEffect(() => {
    if (!visible) setSubmenuOpen(false);
  }, [visible]);

  // 计算菜单位置（避免越界）
  const { left, top, submenuLeft, submenuTop, submenuOnLeft } = useMemo(() => {
    const vw = typeof window !== 'undefined' ? window.innerWidth : 1920;
    const vh = typeof window !== 'undefined' ? window.innerHeight : 1080;
    const menuH = nodeId ? MENU_HEIGHT_NODE : MENU_HEIGHT_PANE;
    const l = Math.min(x, vw - MENU_WIDTH - 8);
    const t = Math.min(y, vh - menuH - 8);

    // 子菜单默认右侧弹出，空间不够则左侧
    const onLeft = l + MENU_WIDTH + SUBMENU_WIDTH + 8 > vw;
    const sLeft = onLeft ? l - SUBMENU_WIDTH - 4 : l + MENU_WIDTH + 4;
    const sTop = Math.min(t, vh - SUBMENU_HEIGHT - 8);
    return {
      left: l,
      top: t,
      submenuLeft: sLeft,
      submenuTop: sTop,
      submenuOnLeft: onLeft,
    };
  }, [x, y, nodeId]);

  // 操作助手：先关菜单，再执行
  const runAndClose = useCallback(
    (fn: () => void) => {
      hideContextMenu();
      requestAnimationFrame(fn);
    },
    [hideContextMenu]
  );

  const handleAddNode = useCallback(
    (type: CanvasNodeType) => {
      runAndClose(() => addNodeAtPosition(type, { x: flowX, y: flowY }));
    },
    [runAndClose, addNodeAtPosition, flowX, flowY]
  );

  const handleUploadClick = useCallback(() => {
    fileInputRef.current?.click();
  }, []);

  const handleFileChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (!file) return;

      const mime = file.type;
      let nodeType: CanvasNodeType = 'image';
      if (mime.startsWith('video/')) nodeType = 'video';
      else if (mime.startsWith('audio/')) nodeType = 'audio';
      else if (mime.startsWith('image/')) nodeType = 'image';

      const url = URL.createObjectURL(file);
      const id = addNodeAtPosition(nodeType, { x: flowX, y: flowY }, {
        label: file.name,
        url,
      });
      updateNodeData(id, { url, label: file.name });

      e.target.value = '';
      hideContextMenu();
    },
    [addNodeAtPosition, updateNodeData, hideContextMenu, flowX, flowY]
  );

  const handlePaste = useCallback(() => {
    runAndClose(() => pasteNode(flowX, flowY));
  }, [runAndClose, pasteNode, flowX, flowY]);

  const handleUndo = useCallback(() => runAndClose(undo), [runAndClose, undo]);
  const handleRedo = useCallback(() => runAndClose(redo), [runAndClose, redo]);

  const handleEditNode = useCallback(() => {
    if (!nodeId) return;
    const id = nodeId;
    runAndClose(() => expandNode(id));
  }, [nodeId, runAndClose, expandNode]);

  const handleCopyNode = useCallback(() => {
    if (!nodeId) return;
    const id = nodeId;
    runAndClose(() => copyNode(id));
  }, [nodeId, runAndClose, copyNode]);

  const handleDisconnect = useCallback(() => {
    if (!nodeId) return;
    const id = nodeId;
    runAndClose(() => disconnectNode(id));
  }, [nodeId, runAndClose, disconnectNode]);

  const handleFullscreenNode = useCallback(() => {
    if (!nodeId) return;
    const id = nodeId;
    runAndClose(() => {
      fitView({ nodes: [{ id }], duration: 400, padding: 0.3 });
    });
  }, [nodeId, runAndClose, fitView]);

  const handleDeleteNode = useCallback(() => {
    if (!nodeId) return;
    const id = nodeId;
    runAndClose(() => removeNode(id));
  }, [nodeId, runAndClose, removeNode]);

  // 子菜单 hover 延迟关闭
  const openSubmenu = () => {
    if (closeTimerRef.current) {
      window.clearTimeout(closeTimerRef.current);
      closeTimerRef.current = null;
    }
    setSubmenuOpen(true);
  };
  const scheduleCloseSubmenu = () => {
    if (closeTimerRef.current) window.clearTimeout(closeTimerRef.current);
    closeTimerRef.current = window.setTimeout(() => setSubmenuOpen(false), 120);
  };

  if (!visible) return null;

  return (
    <>
      {/* 背景遮罩 */}
      <div className="fixed inset-0 z-40" onClick={hideContextMenu} />

      {/* 隐藏的文件选择器 */}
      <input
        ref={fileInputRef}
        type="file"
        accept="image/*,video/*,audio/*"
        className="hidden"
        onChange={handleFileChange}
      />

      {/* 主菜单 */}
      <div
        className="fixed z-50 w-56 rounded-lg border border-zinc-700/60 bg-zinc-800/95 backdrop-blur-sm py-1 shadow-xl shadow-black/30 text-zinc-200 animate-in fade-in-0 zoom-in-95"
        style={{ left, top }}
        onContextMenu={(e) => e.preventDefault()}
      >
        {nodeId ? (
          // ===== 节点右键菜单 =====
          <>
            <MenuRow icon={<Edit3 size={15} />} label="编辑" onClick={handleEditNode} />
            <MenuRow icon={<Copy size={15} />} label="复制" onClick={handleCopyNode} />
            <MenuRow icon={<Unlink size={15} />} label="断开连线" onClick={handleDisconnect} />
            <MenuRow icon={<Maximize2 size={15} />} label="聚焦" onClick={handleFullscreenNode} />
            <Divider />
            <MenuRow
              icon={<Trash2 size={15} />}
              label="删除"
              danger
              onClick={handleDeleteNode}
            />
          </>
        ) : (
          // ===== 画布空白右键菜单 =====
          <>
            <MenuRow icon={<Upload size={15} />} label="上传" onClick={handleUploadClick} />

            {/* 添加节点（含子菜单） */}
            <div
              className="relative"
              onMouseEnter={openSubmenu}
              onMouseLeave={scheduleCloseSubmenu}
            >
              <button
                type="button"
                className={`w-full flex items-center gap-2.5 px-4 py-2.5 text-sm cursor-pointer transition-colors ${
                  submenuOpen ? 'bg-zinc-700/60' : 'hover:bg-zinc-700/60'
                }`}
              >
                <span className="text-zinc-300"><Plus size={15} /></span>
                <span className="flex-1 text-left">添加节点</span>
                <ChevronRight size={14} className="text-zinc-500" />
              </button>
            </div>

            <Divider />
            <MenuRow
              icon={<Undo2 size={15} />}
              label="撤销"
              shortcut="Ctrl+Z"
              onClick={handleUndo}
            />
            <MenuRow
              icon={<Redo2 size={15} />}
              label="重做"
              shortcut="Ctrl+Shift+Z"
              onClick={handleRedo}
            />
            <Divider />
            <MenuRow
              icon={<ClipboardPaste size={15} />}
              label="粘贴"
              shortcut="Ctrl+V"
              onClick={handlePaste}
            />
          </>
        )}
      </div>

      {/* 添加节点 子菜单 */}
      {!nodeId && submenuOpen && (
        <div
          className="fixed z-50 w-50 min-w-[180px] rounded-lg border border-zinc-700/60 bg-zinc-800/95 backdrop-blur-sm py-1 shadow-xl shadow-black/30 text-zinc-200 animate-in fade-in-0 zoom-in-95"
          style={{
            left: submenuLeft,
            top: submenuTop,
            transformOrigin: submenuOnLeft ? 'right top' : 'left top',
          }}
          onMouseEnter={openSubmenu}
          onMouseLeave={scheduleCloseSubmenu}
          onContextMenu={(e) => e.preventDefault()}
        >
          {NODE_ITEMS.map(({ type, icon, label }) => (
            <button
              key={type}
              type="button"
              className="w-full flex items-center gap-3 px-4 py-2.5 text-sm cursor-pointer hover:bg-zinc-700/60 transition-colors text-left"
              onClick={() => handleAddNode(type)}
            >
              <span className="flex h-6 w-6 items-center justify-center rounded-md bg-zinc-700/50 text-zinc-200">
                {icon}
              </span>
              <span>{label}</span>
            </button>
          ))}
        </div>
      )}
    </>
  );
}

// ----- 子组件 -----

function MenuRow({
  icon,
  label,
  shortcut,
  onClick,
  danger = false,
}: {
  icon: ReactNode;
  label: string;
  shortcut?: string;
  onClick: () => void;
  danger?: boolean;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={`w-full flex items-center gap-2.5 px-4 py-2.5 text-sm cursor-pointer transition-colors text-left ${
        danger
          ? 'text-rose-300 hover:bg-rose-500/15 hover:text-rose-200'
          : 'text-zinc-200 hover:bg-zinc-700/60'
      }`}
    >
      <span className={danger ? 'text-rose-300' : 'text-zinc-300'}>{icon}</span>
      <span className="flex-1">{label}</span>
      {shortcut && (
        <span className="text-zinc-500 text-xs tracking-wide">{shortcut}</span>
      )}
    </button>
  );
}

function Divider() {
  return <div className="border-t border-zinc-700/50 my-1" />;
}