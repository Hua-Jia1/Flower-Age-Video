import { useEffect, useRef, useState } from 'react';
import { useChatStore, Message } from './chatStore';
import { useCanvasStore } from '../canvas/store';

export default function ChatPanel() {
  const [input, setInput] = useState('');
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const messages = useChatStore((s) => s.messages);
  const isProcessing = useChatStore((s) => s.isProcessing);
  const sendMessage = useChatStore((s) => s.sendMessage);
  const currentProjectId = useCanvasStore((s) => s.currentProjectId);

  // 监听 Tauri agent-progress 事件
  useEffect(() => {
    let unlisten: (() => void) | undefined;

    const setupListener = async () => {
      try {
        const { listen } = await import('@tauri-apps/api/event');
        unlisten = await listen<{ status: string; progress: number; message: string }>(
          'agent-progress',
          (event) => {
            const { status, message } = event.payload;
            const updateMsg = useChatStore.getState().updateLastAssistantMessage;

            if (status === 'thinking') {
              updateMsg(message || '正在思考...', 'thinking');
            } else if (status === 'executing') {
              updateMsg(message || '正在执行...', 'executing');
            }
            // complete 状态由 sendMessage 中的 response 处理
          }
        );
      } catch {
        // 非 Tauri 环境，忽略
      }
    };

    setupListener();
    return () => {
      unlisten?.();
    };
  }, []);

  // 自动滚动到底部
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  const handleSend = async () => {
    if (!input.trim() || isProcessing || !currentProjectId) return;
    const content = input.trim();
    setInput('');
    await sendMessage(currentProjectId, content);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  return (
    <div className="w-80 h-full border-l bg-background flex flex-col">
      {/* Header */}
      <div className="p-4 border-b flex items-center justify-between">
        <h2 className="font-semibold text-sm">花甲创作</h2>
        {isProcessing && (
          <span className="flex items-center gap-1 text-xs text-blue-500">
            <span className="w-2 h-2 rounded-full bg-blue-500 animate-pulse" />
            处理中
          </span>
        )}
      </div>

      {/* Messages */}
      <div className="flex-1 overflow-y-auto p-3 space-y-3">
        {messages.length === 0 && (
          <div className="text-center text-sm text-muted-foreground mt-8">
            <p className="mb-2">在这里描述你的创作意图</p>
            <p className="text-xs">例如: "生成一张赛博朋克风格城市夜景"</p>
          </div>
        )}
        {messages.map((msg) => (
          <MessageBubble key={msg.id} message={msg} />
        ))}
        <div ref={messagesEndRef} />
      </div>

      {/* Input */}
      <div className="p-3 border-t">
        <div className="flex gap-2">
          <input
            type="text"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder={currentProjectId ? '描述你的创作意图...' : '请先创建项目'}
            disabled={isProcessing || !currentProjectId}
            className="flex-1 px-3 py-2 rounded-md border bg-background text-sm disabled:opacity-50"
          />
          <button
            onClick={handleSend}
            disabled={isProcessing || !input.trim() || !currentProjectId}
            className="px-3 py-2 rounded-md bg-primary text-primary-foreground text-sm font-medium disabled:opacity-50 hover:bg-primary/90 transition-colors"
          >
            发送
          </button>
        </div>
      </div>
    </div>
  );
}

function MessageBubble({ message }: { message: Message }) {
  const isUser = message.role === 'user';
  const isSystem = message.role === 'system';

  if (isSystem) {
    return (
      <div className="text-center text-xs text-muted-foreground py-1">
        {message.content}
      </div>
    );
  }

  return (
    <div className={`flex ${isUser ? 'justify-end' : 'justify-start'}`}>
      <div
        className={`max-w-[85%] rounded-lg px-3 py-2 text-sm ${
          isUser
            ? 'bg-primary text-primary-foreground'
            : 'bg-muted text-foreground'
        }`}
      >
        {/* 状态指示器 */}
        {!isUser && message.status && message.status !== 'complete' && (
          <div className="flex items-center gap-1.5 mb-1 text-xs opacity-70">
            {message.status === 'thinking' && (
              <>
                <span className="w-1.5 h-1.5 rounded-full bg-yellow-500 animate-pulse" />
                思考中
              </>
            )}
            {message.status === 'executing' && (
              <>
                <span className="w-1.5 h-1.5 rounded-full bg-blue-500 animate-pulse" />
                执行中
              </>
            )}
            {message.status === 'error' && (
              <>
                <span className="w-1.5 h-1.5 rounded-full bg-red-500" />
                错误
              </>
            )}
          </div>
        )}
        <p className="whitespace-pre-wrap break-words">{message.content}</p>


      </div>
    </div>
  );
}
