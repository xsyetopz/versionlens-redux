use versionlens_parsers::Dependency;
use versionlens_suggestions::Suggestion;

pub(in crate::presentation::title) fn no_match_title_text(
    _: &Dependency,
    suggestion: &Suggestion,
) -> String {
    suggestion
        .latest
        .as_deref()
        .unwrap_or("no match")
        .to_owned()
}
