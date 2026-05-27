import { create } from 'zustand';
import { startVibeSession, ChatMessage, AgentResponse } from '../lib/tauri';

export interface Message {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: string;
  status?: 'thinking' | 'executing' | 'complete' | 'error';
  agentResponse?: AgentResponse;
}

interface ChatState {
  messages: Message[];
  isProcessing: boolean;
  currentSessionId: string | null;

  // Actions
  sendMessage: (projectId: string, content: string) => Promise<void>;
  addSystemMessage: (content: string) => void;
  updateLastAssistantMessage: (content: string, status?: string) => void;
  clearMessages: () => void;
}

export const useChatStore = create<ChatState>((set, get) => ({
  messages: [],
  isProcessing: false,
  currentSessionId: null,

  sendMessage: async (projectId: string, content: string) => {
    const userMessage: Message = {
      id: crypto.randomUUID(),
      role: 'user',
      content,
      timestamp: new Date().toISOString(),
    };

    // 添加用户消息 + 占位 assistant 消息
    const assistantMessage: Message = {
      id: crypto.randomUUID(),
      role: 'assistant',
      content: '正在思考...',
      timestamp: new Date().toISOString(),
      status: 'thinking',
    };

    set((state) => ({
      messages: [...state.messages, userMessage, assistantMessage],
      isProcessing: true,
    }));

    try {
      // 构建 chat_history（最近 10 条，排除 system 与当前占位消息）
      const chatHistory: ChatMessage[] = get().messages
        .filter((m) => m.role !== 'system' && m.id !== assistantMessage.id)
        .slice(-10)
        .map((m) => ({ role: m.role, content: m.content }));

      const response = await startVibeSession(projectId, content, chatHistory);

      // 更新 assistant 消息
      set((state) => ({
        messages: state.messages.map((m) =>
          m.id === assistantMessage.id
            ? {
                ...m,
                content: response.message || response.summary || '暂无回复',
                status: 'complete' as const,
                agentResponse: response,
              }
            : m
        ),
        isProcessing: false,
        currentSessionId: response.session_id,
      }));
    } catch (error) {
      set((state) => ({
        messages: state.messages.map((m) =>
          m.id === assistantMessage.id
            ? { ...m, content: `错误: ${error}`, status: 'error' as const }
            : m
        ),
        isProcessing: false,
      }));
    }
  },

  addSystemMessage: (content: string) => {
    set((state) => ({
      messages: [
        ...state.messages,
        {
          id: crypto.randomUUID(),
          role: 'system',
          content,
          timestamp: new Date().toISOString(),
        },
      ],
    }));
  },

  updateLastAssistantMessage: (content: string, status?: string) => {
    set((state) => {
      const messages = [...state.messages];
      for (let i = messages.length - 1; i >= 0; i--) {
        if (messages[i].role === 'assistant') {
          messages[i] = {
            ...messages[i],
            content,
            status: status as Message['status'],
          };
          break;
        }
      }
      return { messages };
    });
  },

  clearMessages: () => set({ messages: [], currentSessionId: null }),
}));
