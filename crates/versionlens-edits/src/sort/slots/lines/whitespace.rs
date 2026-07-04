pub(super) fn leading_whitespace_len(line: &str) -> usize {
    line.chars()
        .take_while(|character| character.is_whitespace())
        .count()
}
