use crate::binding::config::NativeSessionConfig;

fn blank_session_config() -> NativeSessionConfig {
    NativeSessionConfig {
        cache_duration_minutes: None,
        enabled_providers: None,
        providers: None,
        suggestion_indicators: None,
        show_vulnerabilities: None,
        show_suggestion_stats: None,
        show_prereleases: false,
        http: None,
    }
}

mod cache;
mod http;
mod indicators;
mod mapping;
mod providers;
