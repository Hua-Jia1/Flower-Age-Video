import { memo } from 'react';
import { Handle, Position } from '@xyflow/react';
import type { NodeProps } from '@xyflow/react';
import { useCanvasStore } from '../store';
import NodeFloatingToolbar from './NodeFloatingToolbar';

const HANDLE_CLS =
  '!bg-zinc-400 hover:!bg-blue-400 !w-2.5 !h-2.5 !opacity-0 group-hover:!opacity-100 transition-opacity duration-200';

const TableIcon = ({ className = 'w-4 h-4 text-zinc-400 shrink-0' }: { className?: string }) => (
  <svg className={className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round">
    <rect x="3" y="4" width="18" height="16" rx="2" />
    <path d="M3 10h18" />
    <path d="M3 15h18" />
    <path d="M9 4v16" />
    <path d="M15 4v16" />
  </svg>
);

interface TableData {
  label?: string;
  columns?: string[];
  rows?: string[][];
}

function TableNode({ id, data, selected }: NodeProps) {
  const updateNodeData = useCanvasStore((s) => s.updateNodeData);
  const d = data as TableData;
  const columns = d.columns ?? ['列1'];
  const rows = d.rows ?? [];

  const handleColChange = (idx: number, value: string) => {
    const next = [...columns];
    next[idx] = value;
    updateNodeData(id, { columns: next });
  };

  const handleCellChange = (r: number, c: number, value: string) => {
    const next = rows.map((row) => [...row]);
    next[r][c] = value;
    updateNodeData(id, { rows: next });
  };

  const addRow = () => {
    const empty = columns.map(() => '');
    updateNodeData(id, { rows: [...rows, empty] });
  };

  const addColumn = () => {
    const nextCols = [...columns, `列${columns.length + 1}`];
    const nextRows = rows.map((row) => [...row, '']);
    updateNodeData(id, { columns: nextCols, rows: nextRows });
  };

  return (
    <div
      className={`group relative rounded-lg border bg-slate-800/95 text-zinc-200 w-[380px] shadow-xl shadow-black/30 backdrop-blur-sm transition-all ${
        selected ? 'ring-1 ring-blue-500/60 border-zinc-500' : 'border-zinc-500/40'
      }`}
    >
      <NodeFloatingToolbar nodeId={id} primaryLabel="生成音频" />
      <div className="flex items-center gap-2 px-4 py-3 border-b border-zinc-700/50">
        <TableIcon />
        <span className="text-sm font-medium tracking-tight">{d.label || '未命名表格'}</span>
        <span className="ml-auto text-[10px] uppercase tracking-[0.18em] text-zinc-500 tabular-nums">
          {rows.length} × {columns.length}
        </span>
      </div>
      <div className="p-3">
        <div className="bg-slate-900/60 rounded-md p-2 overflow-hidden">
          <div className="overflow-x-auto">
            <table className="w-full text-xs border-separate border-spacing-0">
              <thead>
                <tr>
                  {columns.map((col, idx) => (
                    <th key={idx} className="px-2 py-1.5 text-left border-b border-zinc-700/60">
                      <input
                        className="nodrag w-full bg-transparent text-zinc-300 font-medium tracking-tight outline-none placeholder-zinc-600 focus:text-blue-300"
                        value={col}
                        onChange={(e) => handleColChange(idx, e.target.value)}
                        onClick={(e) => e.stopPropagation()}
                        onMouseDown={(e) => e.stopPropagation()}
                      />
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {rows.length === 0 ? (
                  <tr>
                    <td colSpan={columns.length} className="px-2 py-6 text-center text-zinc-500 text-[11px] tracking-wide">
                      暂无数据 — 点击下方 <span className="text-zinc-300">+</span> 添加一行
                    </td>
                  </tr>
                ) : (
                  rows.map((row, r) => (
                    <tr key={r} className="hover:bg-zinc-800/40">
                      {columns.map((_, c) => (
                        <td key={c} className="px-2 py-1.5 border-b border-zinc-800/60">
                          <input
                            className="nodrag w-full bg-transparent text-zinc-300 outline-none placeholder-zinc-600 focus:text-blue-200"
                            value={row[c] ?? ''}
                            onChange={(e) => handleCellChange(r, c, e.target.value)}
                            onClick={(e) => e.stopPropagation()}
                            onMouseDown={(e) => e.stopPropagation()}
                          />
                        </td>
                      ))}
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>
        </div>
        <div className="flex items-center gap-2 mt-2">
          <button
            type="button"
            className="nodrag flex-1 px-2 py-1.5 rounded-md bg-zinc-800/80 hover:bg-zinc-700/80 text-zinc-300 text-xs font-medium border border-zinc-700/60 transition-colors"
            onClick={(e) => { e.stopPropagation(); addRow(); }}
            onMouseDown={(e) => e.stopPropagation()}
          >
            + 添加一行
          </button>
          <button
            type="button"
            className="nodrag px-2 py-1.5 rounded-md bg-zinc-800/80 hover:bg-zinc-700/80 text-zinc-300 text-xs font-medium border border-zinc-700/60 transition-colors"
            onClick={(e) => { e.stopPropagation(); addColumn(); }}
            onMouseDown={(e) => e.stopPropagation()}
          >
            + 列
          </button>
        </div>
      </div>
      <Handle type="target" position={Position.Top} className={HANDLE_CLS} />
      <Handle type="source" position={Position.Bottom} className={HANDLE_CLS} />
      <Handle type="target" position={Position.Left} id="left" className={HANDLE_CLS} />
      <Handle type="source" position={Position.Right} id="right" className={HANDLE_CLS} />
    </div>
  );
}

export default memo(TableNode);