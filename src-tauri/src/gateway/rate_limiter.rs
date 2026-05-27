//! 速率限制器（令牌桶算法）

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;
use std::time::Instant;

/// 速率限制配置
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub global_qps: u32,
    pub global_capacity: u32,
    pub provider_configs: HashMap<String, ProviderRateConfig>,
}

#[derive(Debug, Clone)]
pub struct ProviderRateConfig {
    pub qps: u32,
    pub capacity: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        let mut provider_configs = HashMap::new();
        provider_configs.insert(
            "openai".to_string(),
            ProviderRateConfig {
                qps: 30,
                capacity: 60,
            },
        );
        provider_configs.insert(
            "anthropic".to_string(),
            ProviderRateConfig {
                qps: 20,
                capacity: 40,
            },
        );

        Self {
            global_qps: 100,
            global_capacity: 200,
            provider_configs,
        }
    }
}

/// 令牌桶
struct TokenBucket {
    capacity: u32,
    tokens: AtomicU32,
    refill_rate: u32,
    last_refill: Mutex<Instant>,
}

impl TokenBucket {
    fn new(capacity: u32, refill_rate: u32) -> Self {
        Self {
            capacity,
            tokens: AtomicU32::new(capacity),
            refill_rate,
            last_refill: Mutex::new(Instant::now()),
        }
    }

    fn try_acquire(&self) -> bool {
        // 先尝试补充
        self.refill();

        // 尝试消耗一个 token
        loop {
            let current = self.tokens.load(Ordering::Relaxed);
            if current == 0 {
                return false;
            }
            if self
                .tokens
                .compare_exchange(current, current - 1, Ordering::SeqCst, Ordering::Relaxed)
                .is_ok()
            {
                return true;
            }
        }
    }

    fn refill(&self) {
        let mut last = self.last_refill.lock().unwrap();
        let now = Instant::now();
        let elapsed = now.duration_since(*last).as_secs_f64();
        let refill_amount = (elapsed * self.refill_rate as f64) as u32;

        if refill_amount > 0 {
            let current = self.tokens.load(Ordering::Relaxed);
            let new_tokens = (current + refill_amount).min(self.capacity);
            self.tokens.store(new_tokens, Ordering::Relaxed);
            *last = now;
        }
    }
}

/// 速率限制器
pub struct RateLimiter {
    global_bucket: TokenBucket,
    provider_buckets: HashMap<String, TokenBucket>,
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(RateLimitConfig::default())
    }
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        let global_bucket = TokenBucket::new(config.global_capacity, config.global_qps);

        let mut provider_buckets = HashMap::new();
        for (name, pc) in config.provider_configs {
            provider_buckets.insert(name, TokenBucket::new(pc.capacity, pc.qps));
        }

        Self {
            global_bucket,
            provider_buckets,
        }
    }

    /// 尝试获取令牌
    pub async fn acquire(&self, provider: &str) -> Result<(), String> {
        // 检查全局限制
        if !self.global_bucket.try_acquire() {
            return Err("Global rate limit exceeded".to_string());
        }

        // 检查供应商限制
        if let Some(bucket) = self.provider_buckets.get(provider) {
            if !bucket.try_acquire() {
                // 归还全局 token
                self.global_bucket.tokens.fetch_add(1, Ordering::Relaxed);
                return Err(format!("Rate limit exceeded for provider '{}'", provider));
            }
        }

        Ok(())
    }
}
