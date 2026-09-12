use super::operator_requirement_range;

#[test]
fn operator_requirement_range_keeps_prefix_and_trims_value_bounds() {
    let range = operator_requirement_range("  >=  1.2.3  ", &[">=", ">"]);

    assert_eq!(range.prefix, "  >=  ");
    assert_eq!(range.start, 0);
    assert_eq!(range.end, 11);
}

#[test]
fn operator_requirement_range_uses_trimmed_requirement_when_no_operator_matches() {
    let range = operator_requirement_range("  1.2.3  ", &[">=", ">"]);

    assert_eq!(range.prefix, "");
    assert_eq!(range.start, 2);
    assert_eq!(range.end, 7);
}
