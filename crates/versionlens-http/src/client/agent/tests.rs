use super::cache_key;
use crate::config::HttpConfig;

pub(crate) fn uses_same_agent_cache_key(first: &HttpConfig, second: &HttpConfig) -> bool {
    cache_key(first) == cache_key(second)
}

pub(crate) fn uses_agent_cache(config: &HttpConfig) -> bool {
    cache_key(config).is_some()
}
