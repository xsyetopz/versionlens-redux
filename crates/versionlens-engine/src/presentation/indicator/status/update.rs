use crate::SuggestionIndicators;

pub(crate) fn update_indicator(
    indicators: &SuggestionIndicators,
    has_vulnerabilities: bool,
) -> &str {
    if has_vulnerabilities {
        if indicators.updateable_vulnerable.is_empty() {
            return "⚠️";
        }
        return &indicators.updateable_vulnerable;
    }

    &indicators.updateable
}
