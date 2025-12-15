use serde_json::Value;

#[derive(Clone, Debug)]
pub struct CacheEntry {
    pub value: Value,
    pub stored_at: u64,
}

impl CacheEntry {
    pub fn new(value: Value, stored_at: u64) -> Self {
        Self { value, stored_at }
    }

    pub fn from_string(value: String, stored_at: u64) -> Self {
        Self::new(Value::String(value), stored_at)
    }

    pub fn as_str(&self) -> Option<&str> {
        self.value.as_str()
    }

    pub fn is_expired(&self, ttl_secs: u64) -> bool {
        let now = now_secs();
        now.saturating_sub(self.stored_at) > ttl_secs
    }
}

pub fn now_secs() -> u64 {
    (js_sys::Date::new_0().get_time() / 1000.0) as u64
}
