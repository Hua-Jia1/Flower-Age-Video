import { create } from 'zustand';
import {
  Node,
  Edge,
  NodeChange,
  EdgeChange,
  Connection,
  applyNodeChanges,
  applyEdgeChanges,
  addEdge,
  XYPosition,
} from '@xyflow/react';

// 节点类型定义（精简为 5 种）
export type CanvasNodeType = 'text' | 'table' | 'image' | 'video' | 'audio';

// 右键菜单状态
interface ContextMenuState {
  visible: boolean;
  x: number;
  y: number;
  flowX: number;
  flowY: number;
  nodeId?: string;
}

// 节点数据类型
interface BaseNodeData {
  label: string;
  [key: string]: unknown;
}

// 历史快照
interface HistorySnapshot {
  nodes: Node[];
  edges: Edge[];
}

interface CanvasState {
  // 核心数据
  nodes: Node[];
  edges: Edge[];

  // UI 状态
  selectedNodeId: string | null;
  contextMenu: ContextMenuState;
  expandedNodeId: string | null;

  // 历史（undo/redo）
  history: HistorySnapshot[];
  historyIndex: number;
  _isUndoRedo: boolean;

  // 剪贴板（copy/paste）
  clipboard: Node | null;

  // ReactFlow 事件处理
  onNodesChange: (changes: NodeChange[]) => void;
  onEdgesChange: (changes: EdgeChange[]) => void;
  onConnect: (connection: Connection) => void;

  // 节点操作
  addNode: (node: Node) => void;
  addNodeAtPosition: (type: CanvasNodeType, position: XYPosition, data?: Partial<BaseNodeData>) => string;
  removeNode: (id: string) => void;
  duplicateNode: (id: string) => void;
  updateNodeData: (id: string, data: Record<string, unknown>) => void;

  // 边操作
  removeEdge: (id: string) => void;
  disconnectNode: (nodeId: string) => void;

  // 批量操作
  setNodes: (nodes: Node[]) => void;
  setEdges: (edges: Edge[]) => void;

  // 选中 / 展开
  setSelectedNode: (id: string | null) => void;
  expandNode: (id: string) => void;
  collapseNode: () => void;

  // 右键菜单
  showContextMenu: (x: number, y: number, flowX: number, flowY: number, nodeId?: string) => void;
  hideContextMenu: () => void;

  // 历史
  pushHistory: () => void;
  undo: () => void;
  redo: () => void;

  // 剪贴板
  copyNode: (id: string) => void;
  pasteNode: (x: number, y: number) => void;

  // 持久化
  currentProjectId: string | null;
  setCurrentProject: (id: string) => void;
  loadCanvas: (projectId: string) => Promise<void>;
  saveCanvas: () => Promise<void>;
}

// 生成唯一 ID
let nodeIdCounter = 0;
function generateNodeId(): string {
  return `node_${Date.now()}_${++nodeIdCounter}`;
}

// 节点类型的默认数据
function getDefaultNodeData(type: CanvasNodeType): BaseNodeData {
  switch (type) {
    case 'text':
      return { label: '文本', content: '' };
    case 'table':
      return { label: '表格', columns: ['列1'], rows: [] };
    case 'image':
      return { label: '图像', prompt: '', model: '', width: 1024, height: 1024 };
    case 'video':
      return { label: '视频', prompt: '', duration: 5, resolution: '1080p' };
    case 'audio':
      return { label: '音频', text: '', emotion: '中性', voice: '' };
    default:
      return { label: '节点' };
  }
}

export const useCanvasStore = create<CanvasState>((set, get) => ({
  // 核心数据
  nodes: [],
  edges: [],

  // UI 状态
  selectedNodeId: null,
  contextMenu: { visible: false, x: 0, y: 0, flowX: 0, flowY: 0 },
  expandedNodeId: null,

  // 历史 — 初始空快照确保第一次操作后能 undo 回到空画布
  history: [{ nodes: [], edges: [] }],
  historyIndex: 0,
  _isUndoRedo: false,

  // 剪贴板
  clipboard: null,

  // ReactFlow 事件处理
  onNodesChange: (changes) => {
    set({ nodes: applyNodeChanges(changes, get().nodes) });
    // 节点拖拽结束时记录一次历史（避免每帧都 push）
    const dragEnded = changes.some(
      (c) => c.type === 'position' && (c as { dragging?: boolean }).dragging === false
    );
    if (dragEnded) {
      get().pushHistory();
    }
  },

  onEdgesChange: (changes) => {
    set({ edges: applyEdgeChanges(changes, get().edges) });
  },

  onConnect: (connection) => {
    set({ edges: addEdge({ ...connection, type: 'default' }, get().edges) });
    get().pushHistory();
  },

  // 节点操作
  addNode: (node) => {
    set((state) => ({ nodes: [...state.nodes, node] }));
    get().pushHistory();
  },

  addNodeAtPosition: (type, position, data) => {
    const id = generateNodeId();
    const defaultData = getDefaultNodeData(type);
    const newNode: Node = {
      id,
      type,
      position,
      data: { ...defaultData, ...data },
    };
    set((state) => ({ nodes: [...state.nodes, newNode] }));
    get().pushHistory();
    return id;
  },

  removeNode: (id) => {
    set((state) => ({
      nodes: state.nodes.filter((n) => n.id !== id),
      edges: state.edges.filter((e) => e.source !== id && e.target !== id),
      selectedNodeId: state.selectedNodeId === id ? null : state.selectedNodeId,
      expandedNodeId: state.expandedNodeId === id ? null : state.expandedNodeId,
    }));
    get().pushHistory();
  },

  duplicateNode: (id) => {
    const state = get();
    const node = state.nodes.find((n) => n.id === id);
    if (!node) return;
    const newId = generateNodeId();
    const newNode: Node = {
      ...node,
      id: newId,
      position: { x: node.position.x + 50, y: node.position.y + 50 },
      data: { ...node.data },
    };
    set((s) => ({ nodes: [...s.nodes, newNode] }));
    get().pushHistory();
  },

  updateNodeData: (id, data) => {
    set((state) => ({
      nodes: state.nodes.map((node) =>
        node.id === id ? { ...node, data: { ...node.data, ...data } } : node
      ),
    }));
    get().pushHistory();
  },

  // 边操作
  removeEdge: (id) => {
    set((state) => ({
      edges: state.edges.filter((e) => e.id !== id),
    }));
    get().pushHistory();
  },

  disconnectNode: (nodeId) => {
    set((state) => ({
      edges: state.edges.filter((e) => e.source !== nodeId && e.target !== nodeId),
    }));
    get().pushHistory();
  },

  // 批量操作
  setNodes: (nodes) => set({ nodes }),
  setEdges: (edges) => set({ edges }),

  // 选中 / 展开
  setSelectedNode: (id) => set({ selectedNodeId: id }),
  expandNode: (id) => set({ expandedNodeId: id }),
  collapseNode: () => set({ expandedNodeId: null }),

  // 右键菜单
  showContextMenu: (x, y, flowX, flowY, nodeId) => set({
    contextMenu: { visible: true, x, y, flowX, flowY, nodeId },
  }),

  hideContextMenu: () => set({
    contextMenu: { visible: false, x: 0, y: 0, flowX: 0, flowY: 0 },
  }),

  // 历史
  pushHistory: () => {
    const { nodes, edges, history, historyIndex, _isUndoRedo } = get();
    // undo/redo 触发的变更不再次记录，避免污染历史栈
    if (_isUndoRedo) return;
    const snapshot: HistorySnapshot = {
      nodes: structuredClone(nodes),
      edges: structuredClone(edges),
    };
    const newHistory = history.slice(0, historyIndex + 1);
    newHistory.push(snapshot);
    if (newHistory.length > 30) newHistory.shift();
    set({ history: newHistory, historyIndex: newHistory.length - 1 });
  },

  undo: () => {
    const { history, historyIndex } = get();
    if (historyIndex <= 0) return;
    const prev = history[historyIndex - 1];
    set({
      _isUndoRedo: true,
      nodes: structuredClone(prev.nodes),
      edges: structuredClone(prev.edges),
      historyIndex: historyIndex - 1,
    });
    setTimeout(() => set({ _isUndoRedo: false }), 0);
  },

  redo: () => {
    const { history, historyIndex } = get();
    if (historyIndex >= history.length - 1) return;
    const next = history[historyIndex + 1];
    set({
      _isUndoRedo: true,
      nodes: structuredClone(next.nodes),
      edges: structuredClone(next.edges),
      historyIndex: historyIndex + 1,
    });
    setTimeout(() => set({ _isUndoRedo: false }), 0);
  },

  // 剪贴板
  copyNode: (id) => {
    const node = get().nodes.find((n) => n.id === id);
    if (node) set({ clipboard: structuredClone(node) });
  },

  pasteNode: (x, y) => {
    const { clipboard } = get();
    if (!clipboard) return;
    const newId = crypto.randomUUID();
    const newNode: Node = { ...clipboard, id: newId, position: { x, y } };
    set((state) => ({ nodes: [...state.nodes, newNode] }));
    get().pushHistory();
  },

  // 持久化
  currentProjectId: null,

  setCurrentProject: (id) => set({ currentProjectId: id }),

  loadCanvas: async (projectId) => {
    try {
      const { loadCanvas } = await import('../lib/tauri');
      const data = await loadCanvas(projectId);

      const nodes: Node[] = data.nodes.map((n) => ({
        id: n.id,
        type: n.type || 'text',
        position: n.position,
        data: n.data,
        ...(n.width ? { width: n.width } : {}),
        ...(n.height ? { height: n.height } : {}),
      }));

      const edges: Edge[] = data.edges.map((e) => ({
        id: e.id,
        source: e.source,
        target: e.target,
        type: e.type || 'smoothstep',
      }));

      set({
        nodes,
        edges,
        currentProjectId: projectId,
        history: [],
        historyIndex: -1,
        _isUndoRedo: false,
      });
      // 加载完成后记录初始快照，确保 undo 能回到加载后的初始状态
      get().pushHistory();
    } catch (err) {
      console.error('Failed to load canvas:', err);
    }
  },

  saveCanvas: async () => {
    const state = get();
    if (!state.currentProjectId) return;

    try {
      const { saveCanvas } = await import('../lib/tauri');

      const nodes = state.nodes.map((n) => ({
        id: n.id,
        type: n.type,
        position: n.position,
        data: n.data as Record<string, unknown>,
        width: (n as any).width,
        height: (n as any).height,
      }));

      const edges = state.edges.map((e) => ({
        id: e.id,
        source: e.source,
        target: e.target,
        type: e.type,
      }));

      await saveCanvas(state.currentProjectId, nodes, edges);
    } catch (err) {
      console.error('Failed to save canvas:', err);
    }
  },
}));
