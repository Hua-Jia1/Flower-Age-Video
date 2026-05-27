import { useCallback, useEffect, useRef } from 'react';
import {
  ReactFlow,
  Background,
  MiniMap,
  ConnectionMode,
  ReactFlowProvider,
  useReactFlow,
  type NodeTypes,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import { useCanvasStore } from './store';
import {
  TextNode,
  TableNode,
  ImageNode,
  VideoNode,
  AudioNode,
} from './nodes';
import ContextMenu from './ContextMenu';

// 注册自定义节点类型（精简为 5 种）
const nodeTypes: NodeTypes = {
  text: TextNode,
  table: TableNode,
  image: ImageNode,
  video: VideoNode,
  audio: AudioNode,
};

function CanvasInner() {
  const reactFlowWrapper = useRef<HTMLDivElement>(null);
  const { screenToFlowPosition } = useReactFlow();

  const {
    nodes,
    edges,
    onNodesChange,
    onEdgesChange,
    onConnect,
    setSelectedNode,
    showContextMenu,
    hideContextMenu,
    expandNode,
    collapseNode,
  } = useCanvasStore();

  // 节点点击 — 选中节点
  const onNodeClick = useCallback(
    (_event: React.MouseEvent, node: { id: string }) => {
      setSelectedNode(node.id);
      hideContextMenu();
    },
    [setSelectedNode, hideContextMenu]
  );

  // 节点双击 — 展开节点
  const onNodeDoubleClick = useCallback(
    (_event: React.MouseEvent, node: { id: string }) => {
      expandNode(node.id);
    },
    [expandNode]
  );

  // 双击间隔检测
  const lastClickTime = useRef<number>(0);
  const lastClickPos = useRef<{ x: number; y: number }>({ x: 0, y: 0 });

  // 画布空白点击 — 取消选中、收起展开节点、关闭菜单；双击打开添加节点菜单
  const onPaneClick = useCallback(
    (event: React.MouseEvent) => {
      const now = Date.now();
      const pos = { x: event.clientX, y: event.clientY };
      const dx = Math.abs(pos.x - lastClickPos.current.x);
      const dy = Math.abs(pos.y - lastClickPos.current.y);

      // 检测双击（300ms 内 + 位置接近）
      if (
        now - lastClickTime.current < 300 &&
        dx < 10 && dy < 10
      ) {
        const target = event.target as HTMLElement;
        if (!target.closest('.react-flow__node')) {
          const flow = screenToFlowPosition(pos);
          showContextMenu(pos.x, pos.y, flow.x, flow.y);
        }
      }

      lastClickTime.current = now;
      lastClickPos.current = pos;

      setSelectedNode(null);
      collapseNode();
      hideContextMenu();
    },
    [setSelectedNode, collapseNode, hideContextMenu, showContextMenu, screenToFlowPosition]
  );

  // 画布空白双击 — 打开"添加节点"菜单
  const onWrapperDoubleClick = useCallback(
    (event: React.MouseEvent<HTMLDivElement>) => {
      const target = event.target as HTMLElement;
      // 双击节点内部时，由 onNodeDoubleClick 处理（展开节点），此处跳过
      if (target.closest('.react-flow__node')) return;
      // 仅响应画布面板区域（pane / viewport / background）的双击
      if (
        !target.closest('.react-flow__pane') &&
        !target.closest('.react-flow__viewport') &&
        !target.closest('.react-flow__background')
      ) {
        return;
      }
      const flow = screenToFlowPosition({ x: event.clientX, y: event.clientY });
      showContextMenu(event.clientX, event.clientY, flow.x, flow.y);
    },
    [showContextMenu, screenToFlowPosition]
  );

  // 键盘快捷键：Ctrl+Z 撤销 / Ctrl+Shift+Z 重做 / Ctrl+V 粘贴 / Delete|Backspace 删除选中节点
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement | null;
      // 输入态时不拦截，避免影响节点编辑
      if (target) {
        const tag = target.tagName;
        if (
          tag === 'INPUT' ||
          tag === 'TEXTAREA' ||
          target.isContentEditable
        ) {
          return;
        }
      }

      const { undo, redo, pasteNode } = useCanvasStore.getState();
      if (e.ctrlKey && !e.shiftKey && (e.key === 'z' || e.key === 'Z')) {
        e.preventDefault();
        undo();
      } else if (e.ctrlKey && e.shiftKey && (e.key === 'z' || e.key === 'Z')) {
        e.preventDefault();
        redo();
      } else if (e.ctrlKey && (e.key === 'v' || e.key === 'V')) {
        e.preventDefault();
        pasteNode(0, 0);
      } else if (e.key === 'Delete' || e.key === 'Backspace') {
        // 如果当前焦点在 input/textarea 中则不处理（避免干扰编辑）
        const tag = (e.target as HTMLElement)?.tagName?.toLowerCase();
        if (tag === 'input' || tag === 'textarea' || tag === 'select') return;
        e.preventDefault();
        const { nodes, removeNode } = useCanvasStore.getState();
        const selectedNodes = nodes.filter(n => n.selected);
        selectedNodes.forEach(n => removeNode(n.id));
      }
    };
    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, []);

  return (
    <div ref={reactFlowWrapper} className="w-full h-full" onDoubleClick={onWrapperDoubleClick}>
      <ReactFlow
        nodes={nodes}
        edges={edges}
        nodeTypes={nodeTypes}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onConnect={onConnect}
        onNodeClick={onNodeClick}
        onNodeDoubleClick={onNodeDoubleClick}
        onPaneClick={onPaneClick}
        onPaneContextMenu={(event) => {
          event.preventDefault();
          const { showContextMenu } = useCanvasStore.getState();
          const flowPosition = screenToFlowPosition({ x: event.clientX, y: event.clientY });
          showContextMenu(event.clientX, event.clientY, flowPosition.x, flowPosition.y);
        }}
        onNodeContextMenu={(event, node) => {
          event.preventDefault();
          const { showContextMenu } = useCanvasStore.getState();
          const flowPosition = screenToFlowPosition({ x: event.clientX, y: event.clientY });
          showContextMenu(event.clientX, event.clientY, flowPosition.x, flowPosition.y, node.id);
        }}
        connectionMode={ConnectionMode.Loose}
        fitView
        zoomOnDoubleClick={false}
        panOnDrag={[2]}
        selectionOnDrag={true}
        zoomOnScroll={true}
        panOnScroll={false}
        selectNodesOnDrag={true}
        multiSelectionKeyCode="Shift"
        defaultEdgeOptions={{
          type: 'default',
          animated: true,
          style: { stroke: '#4b5563', strokeWidth: 2 },
        }}
      >
        <Background />
        <MiniMap nodeStrokeWidth={3} zoomable pannable />
      </ReactFlow>
      <ContextMenu />
    </div>
  );
}

// 用 ReactFlowProvider 包裹，确保 useReactFlow hook 可用
export default function Canvas() {
  return (
    <ReactFlowProvider>
      <CanvasInner />
    </ReactFlowProvider>
  );
}
