import { memo, useState, useRef, useEffect, useCallback } from 'react';
import { Handle, Position } from '@xyflow/react';
import type { NodeProps } from '@xyflow/react';
import { useCanvasStore } from '../store';
import NodeFloatingToolbar from './NodeFloatingToolbar';

const HANDLE_CLS =
  '!bg-zinc-400 hover:!bg-blue-400 !w-2.5 !h-2.5 !opacity-0 group-hover:!opacity-100 transition-opacity duration-200';

// 模型列表
const IMAGE_MODELS = [
  '香蕉2',
  '香蕉Pro',
  'Seedream 5.0 Lite',
  'Seedream 4.5',
  'Kling OmniImage O1',
  'Kling v3 Omni (Image)',
  'Midjourney',
  'G Image 2',
];

// 比例选项
const ASPECT_RATIOS = [
  '自适应', '1:1', '16:9', '9:16', '3:4', '4:3',
  '3:2', '2:3', '5:4', '4:5', '21:9',
];

// 清晰度选项
const RESOLUTIONS = ['自适应', '1K', '2K', '4K'];

// 图片图标
const ImageIcon = ({ className = 'w-4 h-4 text-zinc-400 shrink-0' }: { className?: string }) => (
  <svg className={className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round">
    <rect x="3" y="3" width="18" height="18" rx="2" />
    <circle cx="9" cy="9" r="1.6" />
    <path d="m21 15-5-5-9 9" />
  </svg>
);

// 上传图标
const UploadIcon = () => (
  <svg className="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
    <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
    <polyline points="17 8 12 3 7 8" />
    <line x1="12" y1="3" x2="12" y2="15" />
  </svg>
);

// 展开图标
const ExpandIcon = () => (
  <svg className="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
    <polyline points="15 3 21 3 21 9" />
    <polyline points="9 21 3 21 3 15" />
    <line x1="21" y1="3" x2="14" y2="10" />
    <line x1="3" y1="21" x2="10" y2="14" />
  </svg>
);



// 下拉箭头
const ChevronDown = () => (
  <svg className="w-3 h-3 ml-1" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
    <polyline points="6 9 12 15 18 9" />
  </svg>
);

// 比例图标 SVG
const RatioIcon = ({ ratio }: { ratio: string }) => {
  if (ratio === '自适应') {
    return (
      <svg className="w-5 h-5 text-zinc-300" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
        <rect x="4" y="4" width="16" height="16" rx="1" strokeDasharray="3 2" />
      </svg>
    );
  }
  const [w, h] = ratio.split(':').map(Number);
  const maxDim = 14;
  const scale = maxDim / Math.max(w, h);
  const rw = Math.round(w * scale);
  const rh = Math.round(h * scale);
  const x = (20 - rw) / 2 + 2;
  const y = (20 - rh) / 2 + 2;
  return (
    <svg className="w-5 h-5 text-zinc-300" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
      <rect x={x} y={y} width={rw} height={rh} rx="1" />
    </svg>
  );
};

interface ImageNodeData {
  label?: string;
  imageUrl?: string;
  referenceUrls?: string[];
  prompt?: string;
  model?: string;
  aspectRatio?: string;
  resolution?: string;
  // legacy fields
  url?: string;
  referenceUrl?: string;
}

function ImageNode({ id, data, selected }: NodeProps) {
  const updateNodeData = useCanvasStore((s) => s.updateNodeData);
  const d = data as ImageNodeData;

  const [showModelList, setShowModelList] = useState(false);
  const [showSettings, setShowSettings] = useState(false);

  const modelRef = useRef<HTMLDivElement>(null);
  const settingsRef = useRef<HTMLDivElement>(null);
  const modelBtnRef = useRef<HTMLButtonElement>(null);
  const settingsBtnRef = useRef<HTMLButtonElement>(null);
  const refInputRef = useRef<HTMLInputElement>(null);
  const mainInputRef = useRef<HTMLInputElement>(null);

  const currentModel = d.model || '香蕉2';
  const currentRatio = d.aspectRatio || '自适应';
  const currentResolution = d.resolution || '自适应';
  const displayImage = d.imageUrl || d.url;

  // 参考图多张支持（兼容旧 referenceUrl 单张字段）
  const initialRefs = d.referenceUrls || (d.referenceUrl ? [d.referenceUrl] : []);
  const [referencePreviews, setReferencePreviews] = useState<string[]>(initialRefs);

  // 点击外部关闭弹窗
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (showModelList && modelRef.current && !modelRef.current.contains(e.target as Node) && !modelBtnRef.current?.contains(e.target as Node)) {
        setShowModelList(false);
      }
      if (showSettings && settingsRef.current && !settingsRef.current.contains(e.target as Node) && !settingsBtnRef.current?.contains(e.target as Node)) {
        setShowSettings(false);
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [showModelList, showSettings]);

  // 上传主图
  const handleUploadMain = useCallback(() => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = 'image/*';
    input.onchange = () => {
      const file = input.files?.[0];
      if (!file) return;
      const url = URL.createObjectURL(file);
      updateNodeData(id, { imageUrl: url, label: file.name });
    };
    input.click();
  }, [id, updateNodeData]);

  // 上传参考图（多张）
  const handleReferenceUpload = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const files = e.target.files;
    if (!files || files.length === 0) return;
    const newUrls: string[] = [];
    for (let i = 0; i < files.length; i++) {
      newUrls.push(URL.createObjectURL(files[i]));
    }
    const updated = [...referencePreviews, ...newUrls];
    setReferencePreviews(updated);
    updateNodeData(id, { referenceUrls: updated });
    e.target.value = '';
  }, [id, referencePreviews, updateNodeData]);

  // 删除参考图
  const removeReference = useCallback((index: number) => {
    const updated = referencePreviews.filter((_, i) => i !== index);
    setReferencePreviews(updated);
    updateNodeData(id, { referenceUrls: updated });
  }, [id, referencePreviews, updateNodeData]);

  // 选择模型
  const selectModel = useCallback((model: string) => {
    updateNodeData(id, { model });
    setShowModelList(false);
  }, [id, updateNodeData]);

  // 选择比例
  const selectRatio = useCallback((ratio: string) => {
    updateNodeData(id, { aspectRatio: ratio });
  }, [id, updateNodeData]);

  // 选择清晰度
  const selectResolution = useCallback((res: string) => {
    updateNodeData(id, { resolution: res });
  }, [id, updateNodeData]);

  const stop = (e: React.SyntheticEvent) => e.stopPropagation();

  return (
    <div
      className={`group relative rounded-lg border bg-slate-800/95 text-zinc-200 w-[480px] shadow-xl shadow-black/30 backdrop-blur-sm transition-all ${
        selected ? 'ring-1 ring-blue-500/60 border-zinc-500' : 'border-zinc-500/40'
      }`}
    >
      <NodeFloatingToolbar nodeId={id} primaryLabel="生成图片" />

      {/* 顶部标题栏 */}
      <div className="flex items-center gap-2 px-4 py-2.5 border-b border-zinc-700/50">
        <ImageIcon className="w-4 h-4 text-zinc-400" />
        <span className="text-sm font-medium tracking-tight">图片</span>
      </div>

      {/* 主内容区 */}
      <div className="p-3 pb-0">
        {/* 图片预览/上传区 */}
        <div className="relative bg-zinc-900 rounded-lg overflow-hidden border border-zinc-700/40">
          {displayImage ? (
            <div className="relative group/preview">
              <img
                src={displayImage}
                alt={d.label || 'image'}
                className="w-full h-[220px] object-cover bg-black nodrag"
                onClick={stop}
                onMouseDown={stop}
              />
              <button
                type="button"
                className="nodrag absolute bottom-2 right-2 px-2.5 py-1 rounded-md bg-zinc-900/80 hover:bg-zinc-800 text-zinc-200 text-[11px] font-medium border border-zinc-700/60 backdrop-blur-sm opacity-0 group-hover/preview:opacity-100 transition-opacity"
                onClick={(e) => { stop(e); handleUploadMain(); }}
                onMouseDown={stop}
              >
                替换
              </button>
            </div>
          ) : (
            <div
              className="nodrag h-[220px] flex flex-col items-center justify-center gap-3 cursor-pointer hover:bg-zinc-800/50 transition-colors"
              onClick={(e) => { stop(e); handleUploadMain(); }}
              onMouseDown={stop}
            >
              <ImageIcon className="w-10 h-10 text-zinc-600" />
              <button
                type="button"
                className="nodrag flex items-center gap-1.5 px-3 py-1.5 rounded-md bg-zinc-800 hover:bg-zinc-700 text-zinc-300 text-xs font-medium border border-zinc-600/60 transition-colors"
                onClick={(e) => { stop(e); handleUploadMain(); }}
                onMouseDown={stop}
              >
                <UploadIcon />
                上传图片
              </button>
            </div>
          )}

          {/* 放大按钮 - 右上角 */}
          <button
            type="button"
            className="nodrag absolute top-2 right-2 p-1.5 rounded bg-zinc-800/80 hover:bg-zinc-700 text-zinc-400 hover:text-zinc-200 border border-zinc-600/40 transition-colors"
            onClick={stop}
            onMouseDown={stop}
            title="放大"
          >
            <ExpandIcon />
          </button>
        </div>

        {/* 参考图区域 */}
        <div className="flex items-center gap-2 overflow-x-auto py-2 px-3 mt-3 nodrag" onClick={(e) => e.stopPropagation()} onMouseDown={stop}>
          {/* 已上传的缩略图列表 */}
          {referencePreviews.map((url, idx) => (
            <div key={idx} className="relative shrink-0 w-12 h-12 rounded border border-zinc-600 overflow-hidden group/ref">
              <img src={url} className="w-full h-full object-cover" alt={`参考图${idx + 1}`} />
              <button
                type="button"
                className="absolute -top-1 -right-1 w-4 h-4 bg-red-500 rounded-full text-white text-[10px] flex items-center justify-center opacity-0 group-hover/ref:opacity-100 transition-opacity"
                onClick={(e) => { e.stopPropagation(); removeReference(idx); }}
                onMouseDown={stop}
              >×</button>
            </div>
          ))}
          {/* 添加更多按钮 */}
          <button
            type="button"
            className="shrink-0 w-12 h-12 rounded border-2 border-dashed border-emerald-500/60 flex items-center justify-center text-emerald-500/60 hover:border-emerald-400 hover:text-emerald-400 transition-colors"
            onClick={(e) => { e.stopPropagation(); refInputRef.current?.click(); }}
            onMouseDown={stop}
            title="上传参考图"
          >
            <svg className="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
            </svg>
          </button>
        </div>
        <input
          ref={refInputRef}
          type="file"
          accept="image/*"
          multiple
          className="hidden"
          onChange={handleReferenceUpload}
        />

        {/* 提示词输入 */}
        <div className="mt-2 px-3">
          <textarea
            ref={mainInputRef as any}
            className="nodrag w-full bg-transparent rounded-md px-0 py-1 text-sm text-zinc-300 placeholder-zinc-600 outline-none resize-none min-h-[56px]"
            placeholder="描述你想要生成的内容"
            value={d.prompt || ''}
            onChange={(e) => updateNodeData(id, { prompt: e.target.value })}
            onClick={stop}
            onMouseDown={stop}
          />
        </div>
      </div>

      {/* 底部操作栏 */}
      <div className="relative flex items-center gap-2 px-3 py-2.5 border-t border-zinc-700/50 mt-3">
        {/* 模型选择 */}
        <button
          ref={modelBtnRef}
          type="button"
          className="nodrag flex items-center px-2.5 py-1.5 rounded-md bg-zinc-700/60 hover:bg-zinc-600/60 text-xs text-zinc-300 font-medium transition-colors"
          onClick={(e) => { stop(e); setShowModelList(!showModelList); setShowSettings(false); }}
          onMouseDown={stop}
        >
          {currentModel}
          <ChevronDown />
        </button>

        {/* 比例/清晰度选择 */}
        <button
          ref={settingsBtnRef}
          type="button"
          className="nodrag flex items-center px-2.5 py-1.5 rounded-md bg-zinc-700/40 hover:bg-zinc-600/40 text-xs text-zinc-400 font-medium transition-colors"
          onClick={(e) => { stop(e); setShowSettings(!showSettings); setShowModelList(false); }}
          onMouseDown={stop}
        >
          {currentRatio}
          <ChevronDown />
        </button>

        {/* 发送按钮 */}
        <button
          type="button"
          className="nodrag ml-auto px-4 py-1.5 rounded-md bg-zinc-700/60 hover:bg-zinc-600 text-sm text-zinc-200 font-medium transition-colors"
          onClick={stop}
          onMouseDown={stop}
        >
          发送
        </button>

        {/* 模型列表弹窗 */}
        {showModelList && (
          <div
            ref={modelRef}
            className="nodrag absolute bottom-full left-0 mb-2 w-56 bg-zinc-900 border border-zinc-700/60 rounded-lg shadow-xl shadow-black/40 py-1 z-50"
            onClick={stop}
            onMouseDown={stop}
          >
            {IMAGE_MODELS.map((model) => (
              <button
                key={model}
                type="button"
                className={`w-full text-left px-4 py-2.5 text-sm transition-colors ${
                  model === currentModel
                    ? 'text-zinc-100 bg-zinc-800/60'
                    : 'text-zinc-300 hover:bg-zinc-800/40'
                }`}
                onClick={(e) => { stop(e); selectModel(model); }}
                onMouseDown={stop}
              >
                <span className="flex items-center justify-between">
                  {model}
                  {model === currentModel && <span className="text-zinc-400">✓</span>}
                </span>
              </button>
            ))}
          </div>
        )}

        {/* 设置面板弹窗 */}
        {showSettings && (
          <div
            ref={settingsRef}
            className="nodrag absolute bottom-full left-14 mb-2 w-[320px] bg-zinc-900 border border-zinc-700/60 rounded-lg shadow-xl shadow-black/40 p-4 z-50"
            onClick={stop}
            onMouseDown={stop}
          >
            {/* 关闭按钮 */}
            <div className="flex items-center justify-between mb-3">
              <span className="text-sm font-medium text-zinc-200">设置</span>
              <button
                type="button"
                className="nodrag p-1 rounded hover:bg-zinc-800 text-zinc-400 hover:text-zinc-200 transition-colors"
                onClick={(e) => { stop(e); setShowSettings(false); }}
                onMouseDown={stop}
              >
                <svg className="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <line x1="18" y1="6" x2="6" y2="18" />
                  <line x1="6" y1="6" x2="18" y2="18" />
                </svg>
              </button>
            </div>

            {/* 比例 */}
            <div className="mb-4">
              <span className="text-xs text-zinc-400 font-medium mb-2 block">比例</span>
              <div className="grid grid-cols-5 gap-1.5">
                {ASPECT_RATIOS.map((ratio) => (
                  <button
                    key={ratio}
                    type="button"
                    className={`nodrag flex flex-col items-center gap-1 py-2 px-1 rounded-md text-[10px] transition-colors ${
                      ratio === currentRatio
                        ? 'bg-zinc-700 text-zinc-100 ring-1 ring-zinc-500'
                        : 'bg-zinc-800/60 text-zinc-400 hover:bg-zinc-800 hover:text-zinc-300'
                    }`}
                    onClick={(e) => { stop(e); selectRatio(ratio); }}
                    onMouseDown={stop}
                  >
                    <RatioIcon ratio={ratio} />
                    <span>{ratio}</span>
                  </button>
                ))}
              </div>
            </div>

            {/* 清晰度 */}
            <div>
              <span className="text-xs text-zinc-400 font-medium mb-2 block">清晰度</span>
              <div className="grid grid-cols-4 gap-1.5">
                {RESOLUTIONS.map((res) => (
                  <button
                    key={res}
                    type="button"
                    className={`nodrag py-2 rounded-md text-xs font-medium transition-colors ${
                      res === currentResolution
                        ? 'bg-zinc-700 text-zinc-100 ring-1 ring-zinc-500'
                        : 'bg-zinc-800/60 text-zinc-400 hover:bg-zinc-800 hover:text-zinc-300'
                    }`}
                    onClick={(e) => { stop(e); selectResolution(res); }}
                    onMouseDown={stop}
                  >
                    {res}
                  </button>
                ))}
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Handles */}
      <Handle type="target" position={Position.Top} className={HANDLE_CLS} />
      <Handle type="source" position={Position.Bottom} className={HANDLE_CLS} />
      <Handle type="target" position={Position.Left} id="left" className={HANDLE_CLS} />
      <Handle type="source" position={Position.Right} id="right" className={HANDLE_CLS} />
    </div>
  );
}

export default memo(ImageNode);
