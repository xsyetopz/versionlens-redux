use super::{line_range, offset_range};

#[test]
fn offset_ranges_count_utf16_code_units_like_vscode_position_at() {
    let text = "😀{\"dependencies\":{\"left-pad\":\"1.0.0\"}}\n";
    let name_start = text.find("left-pad").expect("dependency name");
    let range = offset_range(text, name_start, name_start + "left-pad".len());

    assert_eq!(range.start.line, 0);
    assert_eq!(range.start.character, 20);
    assert_eq!(range.end.character, 28);
}

#[test]
fn line_ranges_count_utf16_code_units_like_vscode_position_at() {
    let line = "😀 FROM node:20";
    let name_start = line.find("node").expect("image name");
    let range = line_range(0, line, name_start, name_start + "node".len());

    assert_eq!(range.start.line, 0);
    assert_eq!(range.start.character, 8);
    assert_eq!(range.end.character, 12);
}
