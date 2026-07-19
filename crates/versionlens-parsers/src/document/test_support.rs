use versionlens_model::Range;
pub(crate) fn extract_range(text: &str, range: Range) -> &str {
    let line = text.lines().nth(range.start.line as usize).unwrap_or("");
    let start = utf16_character_to_byte(line, range.start.character);
    let end = utf16_character_to_byte(line, range.end.character);
    &line[start..end]
}

fn utf16_character_to_byte(line: &str, character: u32) -> usize {
    let target = character as usize;
    let mut units = 0;
    for (offset, value) in line.char_indices() {
        if units >= target {
            return offset;
        }
        units += value.len_utf16();
    }
    line.len()
}
