use super::blank_session_config;
use crate::model::config::NativeSessionConfig;
use crate::model::config::NativeSuggestionIndicators;

fn blank_indicators() -> NativeSuggestionIndicators {
    NativeSuggestionIndicators {
        latest: None,
        satisfies_latest: None,
        directory: None,
        error: None,
        no_match: None,
        matched: None,
        updateable: None,
        updateable_vulnerable: None,
        build: None,
    }
}

#[test]
fn partial_indicator_config_keeps_core_defaults() {
    let config = NativeSessionConfig {
        suggestion_indicators: Some(NativeSuggestionIndicators {
            updateable: Some("U".to_owned()),
            ..blank_indicators()
        }),
        ..blank_session_config()
    }
    .into_core();

    assert_eq!(config.suggestion_indicators.latest, "\u{1F7E2}");
    assert_eq!(config.suggestion_indicators.updateable, "U");
    assert_eq!(config.suggestion_indicators.directory, "\u{1F4C1}");
    assert_eq!(config.suggestion_indicators.error, "\u{1F534}");
    assert_eq!(config.suggestion_indicators.matched, "\u{1F7E1}");
    assert_eq!(config.suggestion_indicators.build, "\u{224C} ");
    assert_eq!(
        config.suggestion_indicators.updateable_vulnerable,
        "\u{26A0}\u{FE0F}"
    );
}

#[test]
fn directory_indicator_maps_to_core() {
    let config = NativeSessionConfig {
        suggestion_indicators: Some(NativeSuggestionIndicators {
            directory: Some("D".to_owned()),
            ..blank_indicators()
        }),
        ..blank_session_config()
    }
    .into_core();

    assert_eq!(config.suggestion_indicators.directory, "D");
}
