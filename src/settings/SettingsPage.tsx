import { useEffect, useState } from 'react';
import { listApiKeys, saveApiKey, deleteApiKey, testApiKey } from '../lib/tauri';

// 供应商类型定义
type ProviderType = 'llm' | 'image' | 'video' | 'audio' | 'music';

interface ProviderConfig {
  type: ProviderType;
  name: string;
  description: string;
  placeholder: string;
  needsBaseUrl: boolean;
  defaultModel: string;
}

const PROVIDER_CONFIGS: ProviderConfig[] = [
  {
    type: 'llm',
    name: 'LLM（语言模型）',
    description: 'GPT / Claude 等对话模型，用于剧本生成、意图分类等',
    placeholder: 'sk-...',
    needsBaseUrl: true,
    defaultModel: 'gpt-4o-mini',
  },
  {
    type: 'image',
    name: '图片生成',
    description: 'DALL-E、Stable Diffusion 等图片生成模型',
    placeholder: 'sk-...',
    needsBaseUrl: true,
    defaultModel: 'dall-e-3',
  },
  {
    type: 'video',
    name: '视频生成',
    description: 'Sora、Runway 等视频生成模型',
    placeholder: 'sk-...',
    needsBaseUrl: true,
    defaultModel: 'video-model',
  },
  {
    type: 'audio',
    name: '音频 / 配音',
    description: 'TTS 语音合成模型',
    placeholder: 'sk-...',
    needsBaseUrl: true,
    defaultModel: 'tts-1',
  },
  {
    type: 'music',
    name: '音乐生成',
    description: '音乐生成模型',
    placeholder: 'sk-...',
    needsBaseUrl: true,
    defaultModel: 'music-gen',
  },
];

interface ApiKeyInfo {
  id: string;
  provider: string;
  label: string;
  base_url: string;
  model: string;
  is_active: boolean;
  created_at: string;
}

export default function SettingsPage() {
  const [keys, setKeys] = useState<ApiKeyInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [inputValues, setInputValues] = useState<Record<ProviderType, string>>({} as Record<ProviderType, string>);
  const [baseUrlValues, setBaseUrlValues] = useState<Record<ProviderType, string>>({} as Record<ProviderType, string>);
  const [modelValues, setModelValues] = useState<Record<ProviderType, string>>({} as Record<ProviderType, string>);
  const [testStatus, setTestStatus] = useState<Record<ProviderType, 'idle' | 'testing' | 'success' | 'failed'>>({} as Record<ProviderType, 'idle' | 'testing' | 'success' | 'failed'>);
  const [saving, setSaving] = useState<Record<ProviderType, boolean>>({} as Record<ProviderType, boolean>);

  useEffect(() => {
    loadKeys();
  }, []);

  const loadKeys = async () => {
    try {
      const result = await listApiKeys();
      setKeys(result);
      // 回填已有配置到输入框
      const urlMap: Record<string, string> = {};
      const modelMap: Record<string, string> = {};
      result.forEach((k) => {
        if (k.is_active) {
          urlMap[k.provider] = k.base_url;
          modelMap[k.provider] = k.model;
        }
      });
      setBaseUrlValues(urlMap as Record<ProviderType, string>);
      setModelValues(modelMap as Record<ProviderType, string>);
    } catch (e) {
      console.error('Failed to load API keys:', e);
    } finally {
      setLoading(false);
    }
  };

  const handleSave = async (providerType: ProviderType) => {
    const key = inputValues[providerType];
    if (!key?.trim()) return;

    const config = PROVIDER_CONFIGS.find((p) => p.type === providerType);
    const baseUrl = (baseUrlValues[providerType] || '').trim();
    const model = (modelValues[providerType] || config?.defaultModel || '').trim();

    if (!baseUrl) {
      alert('请填写 API 地址');
      return;
    }

    setSaving((s) => ({ ...s, [providerType]: true }));
    try {
      await saveApiKey(providerType, 'default', key.trim(), baseUrl, model);
      setInputValues((v) => ({ ...v, [providerType]: '' }));
      await loadKeys();
    } catch (e) {
      console.error('Failed to save key:', e);
    } finally {
      setSaving((s) => ({ ...s, [providerType]: false }));
    }
  };

  const handleDelete = async (providerType: ProviderType) => {
    try {
      await deleteApiKey(providerType, 'default');
      await loadKeys();
      setTestStatus((s) => ({ ...s, [providerType]: 'idle' }));
    } catch (e) {
      console.error('Failed to delete key:', e);
    }
  };

  const handleTest = async (providerType: ProviderType) => {
    const key = inputValues[providerType];
    if (!key?.trim()) return;

    const baseUrl = (baseUrlValues[providerType] || '').trim();
    const model = (modelValues[providerType] || '').trim();

    if (!baseUrl) {
      setTestStatus((s) => ({ ...s, [providerType]: 'failed' }));
      return;
    }

    setTestStatus((s) => ({ ...s, [providerType]: 'testing' }));
    try {
      const success = await testApiKey(providerType, key.trim(), baseUrl, model);
      setTestStatus((s) => ({ ...s, [providerType]: success ? 'success' : 'failed' }));
    } catch {
      setTestStatus((s) => ({ ...s, [providerType]: 'failed' }));
    }
  };

  const getKeyForProvider = (providerType: ProviderType) => {
    return keys.find((k) => k.provider === providerType && k.is_active);
  };

  if (loading) {
    return (
      <div className="p-8 flex items-center justify-center">
        <span className="text-muted-foreground">加载中...</span>
      </div>
    );
  }

  return (
    <div className="p-8 max-w-2xl mx-auto">
      <h1 className="text-2xl font-bold mb-2">设置</h1>
      <p className="text-sm text-muted-foreground mb-8">
        配置 AI 供应商的 API Key，密钥将以明文形式存储在本地数据库中。
      </p>

      <div className="space-y-6">
        {PROVIDER_CONFIGS.map((config) => {
          const existingKey = getKeyForProvider(config.type);
          const status = testStatus[config.type] || 'idle';
          const isSaving = saving[config.type] || false;

          return (
            <div key={config.type} className="border rounded-lg p-5">
              {/* Header */}
              <div className="flex items-center justify-between mb-3">
                <div>
                  <h3 className="font-semibold text-base">{config.name}</h3>
                  <p className="text-xs text-muted-foreground">{config.description}</p>
                </div>
                <span
                  className={`text-xs px-2 py-1 rounded-full ${
                    existingKey
                      ? 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400'
                      : 'bg-gray-100 text-gray-500 dark:bg-gray-800 dark:text-gray-400'
                  }`}
                >
                  {existingKey ? '已配置' : '未配置'}
                </span>
              </div>

              {/* 已有 Key 时显示信息 */}
              {existingKey && (
                <div className="flex items-center gap-2 mb-3 p-2 bg-muted/50 rounded text-sm">
                  <span className="text-muted-foreground flex-1 truncate">
                    已配置（{new Date(existingKey.created_at).toLocaleDateString()}）
                    {existingKey.base_url && (
                      <span className="ml-2 text-xs opacity-70">→ {existingKey.base_url}</span>
                    )}
                    {existingKey.model && (
                      <span className="ml-2 text-xs opacity-70">模型: {existingKey.model}</span>
                    )}
                  </span>
                  <button
                    onClick={() => handleDelete(config.type)}
                    className="text-xs text-red-500 hover:text-red-700 px-2 py-1 rounded hover:bg-red-50 dark:hover:bg-red-900/20"
                  >
                    删除
                  </button>
                </div>
              )}

              {/* API 地址 */}
              <div className="mb-2">
                <label className="text-xs text-muted-foreground mb-1 block">API 地址</label>
                <input
                  type="text"
                  value={baseUrlValues[config.type] || ''}
                  onChange={(e) =>
                    setBaseUrlValues((v) => ({ ...v, [config.type]: e.target.value }))
                  }
                  placeholder="https://api.openai.com/v1"
                  className="w-full px-3 py-2 rounded-md border bg-background text-sm font-mono"
                />
              </div>

              {/* 模型名称 */}
              <div className="mb-2">
                <label className="text-xs text-muted-foreground mb-1 block">模型名称</label>
                <input
                  type="text"
                  value={modelValues[config.type] || ''}
                  onChange={(e) =>
                    setModelValues((v) => ({ ...v, [config.type]: e.target.value }))
                  }
                  placeholder={config.defaultModel}
                  className="w-full px-3 py-2 rounded-md border bg-background text-sm font-mono"
                />
              </div>

              {/* 输入新 Key */}
              <div className="flex gap-2">
                <input
                  type="password"
                  value={inputValues[config.type] || ''}
                  onChange={(e) => setInputValues((v) => ({ ...v, [config.type]: e.target.value }))}
                  placeholder={existingKey ? '输入新 Key 替换...' : config.placeholder}
                  className="flex-1 px-3 py-2 rounded-md border bg-background text-sm"
                />
                <button
                  onClick={() => handleTest(config.type)}
                  disabled={!inputValues[config.type]?.trim()}
                  className="px-3 py-2 rounded-md border text-sm disabled:opacity-50 hover:bg-muted transition-colors"
                >
                  {status === 'testing' ? '测试中...' : '测试'}
                </button>
                <button
                  onClick={() => handleSave(config.type)}
                  disabled={!inputValues[config.type]?.trim() || isSaving}
                  className="px-3 py-2 rounded-md bg-primary text-primary-foreground text-sm disabled:opacity-50 hover:bg-primary/90 transition-colors"
                >
                  {isSaving ? '保存中...' : '保存'}
                </button>
              </div>

              {/* 测试状态 */}
              {status !== 'idle' && status !== 'testing' && (
                <div
                  className={`mt-2 text-xs ${
                    status === 'success' ? 'text-green-600' : 'text-red-500'
                  }`}
                >
                  {status === 'success' ? '连接测试成功' : '连接测试失败，请检查 Key、地址与模型是否正确'}
                </div>
              )}
            </div>
          );
        })}
      </div>

      {/* 安全说明 */}
      <div className="mt-8 p-4 border rounded-lg bg-muted/30">
        <h4 className="text-sm font-medium mb-2">说明</h4>
        <ul className="text-xs text-muted-foreground space-y-1">
          <li>API Key 以明文形式存储在本地 SQLite 数据库中（开发阶段，未启用加密）</li>
          <li>API 请求直接从您的电脑发送到供应商服务器，无中间代理</li>
          <li>每个类型均可独立配置，支持中转站或官方 API</li>
          <li>模型名称需与供应商实际支持的模型名一致</li>
        </ul>
      </div>
    </div>
  );
}