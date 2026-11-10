pub(crate) struct RequirementRange {
    pub(crate) prefix: String,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

pub(crate) fn operator_requirement_range(
    requirement: &str,
    operators: &[&str],
) -> RequirementRange {
    let trimmed_start = requirement.len() - requirement.trim_start().len();
    let trimmed = &requirement[trimmed_start..];
    for operator in operators {
        if let Some(rest) = trimmed.strip_prefix(operator) {
            let spaces = rest.len() - rest.trim_start().len();
            let start = trimmed_start + operator.len() + spaces;
            return RequirementRange {
                prefix: requirement[..start].to_owned(),
                start: 0,
                end: requirement.trim_end().len(),
            };
        }
    }

    RequirementRange {
        prefix: "".to_owned(),
        start: trimmed_start,
        end: requirement.trim_end().len(),
    }
}

#[cfg(test)]
mod tests {
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
}
