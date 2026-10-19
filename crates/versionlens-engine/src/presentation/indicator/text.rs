pub(in crate::presentation) fn with_indicator(indicator: &str, title: String) -> String {
    if indicator.is_empty() {
        return title;
    }

    let separator = if cfg!(windows) { "" } else { " " };
    format!("{indicator}{separator}{title}").trim().to_owned()
}
