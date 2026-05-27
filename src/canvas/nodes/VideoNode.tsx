import { memo } from 'react';
import { Handle, Position } from '@xyflow/react';
import type { NodeProps } from '@xyflow/react';
import { useCanvasStore } from '../store';
import NodeFloatingToolbar from './NodeFloatingToolbar';

const HANDLE_CLS =
  '!bg-zinc-400 hover:!bg-blue-400 !w-2.5 !h-2.5 !opacity-0 group-hover:!opacity-100 transition-opacity duration-200';

const VideoIcon = ({ className = 'w-4 h-4 text-zinc-400 shrink-0' }: { className?: string }) => (
  <svg className={className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round">
    <rect x="2" y="6" width="14" height="12" rx="2" />
    <path d="m22 8-6 4 6 4z" />
  </svg>
);

const PlayIcon = ({ className = 'w-6 h-6' }: { className?: string }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M8 5v14l11-7z" />
  </svg>
);

interface VideoData {
  label?: string;
  url?: string;
  prompt?: string;
  duration?: number;
  resolution?: string;
}

function VideoNode({ id, data, selected }: NodeProps) {
  const updateNodeData = useCanvasStore((s) => s.updateNodeData);
  const d = data as VideoData;

  return (
    <div
      className={`group relative rounded-lg border bg-slate-800/95 text-zinc-200 w-[380px] shadow-xl shadow-black/30 backdrop-blur-sm transition-all ${
        selected ? 'ring-1 ring-blue-500/60 border-zinc-500' : 'border-zinc-500/40'
      }`}
    >
      <NodeFloatingToolbar nodeId={id} primaryLabel="生成音频" />
      <div className="flex items-center gap-2 px-4 py-3 border-b border-zinc-700/50">
        <VideoIcon />
        <span className="text-sm font-medium tracking-tight">{d.label || '未命名视频'}</span>
        <span className="ml-auto text-[10px] uppercase tracking-[0.18em] text-zinc-500">Video</span>
      </div>
      <div className="p-3">
        <div className="relative bg-slate-900/60 rounded-md overflow-hidden border border-zinc-800/60 h-[210px]">
          {d.url ? (
            <video src={d.url} className="w-full h-full object-cover bg-black" controls
              onClick={(e) => e.stopPropagation()} onMouseDown={(e) => e.stopPropagation()} />
          ) : (
            <>
              <div className="absolute inset-0"
                style={{ background: 'radial-gradient(ellipse at center, rgba(59,130,246,0.08), transparent 60%), repeating-linear-gradient(135deg, rgba(255,255,255,0.015) 0 2px, transparent 2px 8px)' }} />
              <div className="relative z-10 h-full flex flex-col items-center justify-center gap-3">
                <button type="button"
                  className="nodrag w-14 h-14 rounded-full bg-zinc-100/95 hover:bg-white text-slate-900 flex items-center justify-center shadow-lg shadow-black/40 transition-transform hover:scale-105"
                  onClick={(e) => e.stopPropagation()} onMouseDown={(e) => e.stopPropagation()}>
                  <PlayIcon className="w-6 h-6 ml-1" />
                </button>
                <span className="text-[11px] uppercase tracking-[0.2em] text-zinc-500">尚未生成</span>
              </div>
            </>
          )}
          <div className="absolute bottom-2 left-2 right-2 flex items-center justify-between text-[10px] uppercase tracking-[0.18em] text-zinc-300 pointer-events-none">
            <span className="px-1.5 py-0.5 rounded bg-black/60 backdrop-blur-sm tabular-nums">
              {(d.duration ?? 5).toString().padStart(2, '0')}s
            </span>
            <span className="px-1.5 py-0.5 rounded bg-black/60 backdrop-blur-sm">
              {d.resolution || '1080p'}
            </span>
          </div>
        </div>
        <input
          className="nodrag w-full mt-2 bg-slate-900/60 rounded-md px-3 py-2 text-xs text-zinc-300 placeholder-zinc-600 border border-zinc-800/60 outline-none focus:ring-1 focus:ring-blue-500/40"
          placeholder="描述这个镜头…"
          value={d.prompt || ''}
          onChange={(e) => updateNodeData(id, { prompt: e.target.value })}
          onClick={(e) => e.stopPropagation()} onMouseDown={(e) => e.stopPropagation()}
        />
      </div>
      <Handle type="target" position={Position.Top} className={HANDLE_CLS} />
      <Handle type="source" position={Position.Bottom} className={HANDLE_CLS} />
      <Handle type="target" position={Position.Left} id="left" className={HANDLE_CLS} />
      <Handle type="source" position={Position.Right} id="right" className={HANDLE_CLS} />
    </div>
  );
}

export default memo(VideoNode);