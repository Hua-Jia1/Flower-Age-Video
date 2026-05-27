import { useEffect, useRef } from 'react';
import { useCanvasStore } from '../canvas/store';

/**
 * 自动保存 hook — 画布变更后防抖 2 秒自动保存
 */
export function useAutoSave() {
  const nodes = useCanvasStore((s) => s.nodes);
  const edges = useCanvasStore((s) => s.edges);
  const currentProjectId = useCanvasStore((s) => s.currentProjectId);
  const saveCanvas = useCanvasStore((s) => s.saveCanvas);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const isInitialLoad = useRef(true);

  useEffect(() => {
    // 跳过初始加载触发的保存
    if (isInitialLoad.current) {
      isInitialLoad.current = false;
      return;
    }

    if (!currentProjectId) return;

    // 防抖 2 秒（减少序列化 I/O 频率）
    if (timerRef.current) {
      clearTimeout(timerRef.current);
    }
    timerRef.current = setTimeout(() => {
      saveCanvas();
    }, 2000);

    return () => {
      if (timerRef.current) {
        clearTimeout(timerRef.current);
      }
    };
  }, [nodes, edges, currentProjectId, saveCanvas]);
}