import { memo } from 'react';
import { Handle, Position } from '@xyflow/react';
import type { NodeProps } from '@xyflow/react';
import { useCanvasStore } from '../store';
import NodeFloatingToolbar from './NodeFloatingToolbar';

const HANDLE_CLS =
  '!bg-zinc-400 hover:!bg-blue-400 !w-2.5 !h-2.5 !opacity-0 group-hover:!opacity-100 transition-opacity duration-200';

const DocIcon = ({ className = 'w-4 h-4 text-zinc-400 shrink-0' }: { className?: string }) => (
  <svg className={className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round">
    <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
    <polyline points="14 2 14 8 20 8" />
    <line x1="16" y1="13" x2="8" y2="13" />
    <line x1="16" y1="17" x2="8" y2="17" />
  </svg>
);

function TextNode({ id, data, selected }: NodeProps) {
  const updateNodeData = useCanvasStore((s) => s.updateNodeData);
  const d = data as { label?: string; content?: string };

  return (
    <div
      className={`group relative rounded-lg border bg-slate-800/95 text-zinc-200 w-[380px] shadow-xl shadow-black/30 backdrop-blur-sm transition-all ${
        selected ? 'ring-1 ring-blue-500/60 border-zinc-500' : 'border-zinc-500/40'
      }`}
    >
      <NodeFloatingToolbar nodeId={id} primaryLabel="生成音频" />
      <div className="flex items-center gap-2 px-4 py-3 border-b border-zinc-700/50">
        <DocIcon />
        <span className="text-sm font-medium tracking-tight">{d.label || 'Untitled.md'}</span>
        <span className="ml-auto text-[10px] uppercase tracking-[0.18em] text-zinc-500">Markdown</span>
      </div>
      <div className="p-3">
        <textarea
          className="nodrag w-full min-h-[200px] bg-slate-900/60 rounded-md p-3 text-sm leading-relaxed text-zinc-300 placeholder-zinc-600 border-none outline-none resize-y focus:ring-1 focus:ring-blue-500/40"
          placeholder="开始书写…"
          value={d.content || ''}
          onChange={(e) => updateNodeData(id, { content: e.target.value })}
          onClick={(e) => e.stopPropagation()}
          onMouseDown={(e) => e.stopPropagation()}
        />
      </div>
      <Handle type="target" position={Position.Top} className={HANDLE_CLS} />
      <Handle type="source" position={Position.Bottom} className={HANDLE_CLS} />
      <Handle type="target" position={Position.Left} id="left" className={HANDLE_CLS} />
      <Handle type="source" position={Position.Right} id="right" className={HANDLE_CLS} />
    </div>
  );
}

export default memo(TextNode);