import { memo, useMemo } from 'react';
import { Handle, Position } from '@xyflow/react';
import type { NodeProps } from '@xyflow/react';
import { useCanvasStore } from '../store';
import NodeFloatingToolbar from './NodeFloatingToolbar';

const HANDLE_CLS =
  '!bg-zinc-400 hover:!bg-blue-400 !w-2.5 !h-2.5 !opacity-0 group-hover:!opacity-100 transition-opacity duration-200';

const AudioIcon = ({ className = 'w-4 h-4 text-zinc-400 shrink-0' }: { className?: string }) => (
  <svg className={className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round">
    <path d="M3 12v0" />
    <path d="M7 8v8" />
    <path d="M11 5v14" />
    <path d="M15 9v6" />
    <path d="M19 7v10" />
  </svg>
);

const PlayIcon = ({ className = 'w-4 h-4' }: { className?: string }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path d="M8 5v14l11-7z" />
  </svg>
);

interface AudioData {
  label?: string;
  url?: string;
  text?: string;
  voice?: string;
  emotion?: string;
}

function useWaveform(count: number, seed = 7): number[] {
  return useMemo(() => {
    const out: number[] = [];
    let s = seed;
    for (let i = 0; i < count; i++) {
      s = (s * 9301 + 49297) % 233280;
      const r = s / 233280;
      const center = Math.sin((i / count) * Math.PI);
      out.push(0.18 + r * 0.45 + center * 0.4);
    }
    return out;
  }, [count, seed]);
}

function AudioNode({ id, data, selected }: NodeProps) {
  const updateNodeData = useCanvasStore((s) => s.updateNodeData);
  const d = data as AudioData;
  const bars = useWaveform(48);

  return (
    <div
      className={`group relative rounded-lg border bg-slate-800/95 text-zinc-200 w-[380px] shadow-xl shadow-black/30 backdrop-blur-sm transition-all ${
        selected ? 'ring-1 ring-blue-500/60 border-zinc-500' : 'border-zinc-500/40'
      }`}
    >
      <NodeFloatingToolbar nodeId={id} primaryLabel="生成文本" />
      <div className="flex items-center gap-2 px-4 py-3 border-b border-zinc-700/50">
        <AudioIcon />
        <span className="text-sm font-medium tracking-tight">{d.label || '未命名音频'}</span>
        <span className="ml-auto text-[10px] uppercase tracking-[0.18em] text-zinc-500">
          {d.emotion || 'Audio'}
        </span>
      </div>
      <div className="p-3">
        <div className="bg-slate-900/60 rounded-md p-3 border border-zinc-800/60">
          <div className="h-[88px] flex items-center justify-center gap-[3px]">
            {bars.map((h, i) => (
              <span key={i} className="block w-[3px] rounded-full bg-gradient-to-t from-blue-500/70 to-cyan-300/80"
                style={{ height: `${Math.round(h * 100)}%`, opacity: 0.55 + h * 0.45 }} />
            ))}
          </div>
          <div className="mt-3 flex items-center gap-3">
            <button type="button"
              className="nodrag w-9 h-9 rounded-full bg-zinc-100/95 hover:bg-white text-slate-900 flex items-center justify-center shadow-md shadow-black/30 transition-transform hover:scale-105"
              onClick={(e) => e.stopPropagation()} onMouseDown={(e) => e.stopPropagation()}>
              <PlayIcon className="w-4 h-4 ml-0.5" />
            </button>
            <div className="flex-1 h-1 rounded-full bg-zinc-700/60 overflow-hidden">
              <div className="h-full w-[18%] bg-gradient-to-r from-blue-500 to-cyan-300" />
            </div>
            <span className="text-[10px] uppercase tracking-[0.18em] text-zinc-500 tabular-nums">
              00:00 / 00:00
            </span>
          </div>
        </div>
        <textarea
          className="nodrag w-full mt-2 bg-slate-900/60 rounded-md px-3 py-2 text-xs leading-relaxed text-zinc-300 placeholder-zinc-600 border border-zinc-800/60 outline-none resize-y min-h-[72px] focus:ring-1 focus:ring-blue-500/40"
          placeholder="输入要朗读的文本…"
          value={d.text || ''}
          onChange={(e) => updateNodeData(id, { text: e.target.value })}
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

export default memo(AudioNode);