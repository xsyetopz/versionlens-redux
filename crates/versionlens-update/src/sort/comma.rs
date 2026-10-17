pub(super) fn match_trailing_comma(target: &str, replacement: &str) -> String {
    match (has_trailing_comma(target), has_trailing_comma(replacement)) {
        (true, false) => add_trailing_comma(replacement),
        (false, true) => remove_trailing_comma(replacement),
        _ => replacement.to_owned(),
    }
}

fn has_trailing_comma(line: &str) -> bool {
    line.trim_end().ends_with(',')
}

fn add_trailing_comma(line: &str) -> String {
    let mut line = line.to_owned();
    line.insert(line.trim_end().len(), ',');
    line
}

fn remove_trailing_comma(line: &str) -> String {
    let mut line = line.to_owned();
    let end = line.trim_end().len();
    line.remove(end - 1);
    line
}
