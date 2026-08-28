use super::requirement_satisfies;

#[test]
fn exclusive_ordered_comparisons_exclude_same_base_derived_releases() {
    for (requirement, candidate, expected) in [
        (">1.0", "1.0.post1", false),
        (">1.0.post1", "1.0.post2", true),
        ("<1.0", "1.0rc1", false),
        ("<1.0rc1", "1.0b1", true),
    ] {
        assert_eq!(
            requirement_satisfies(requirement, candidate),
            Some(expected),
            "{requirement} against {candidate}",
        );
    }
}
