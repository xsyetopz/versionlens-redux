use crate::NpmRegistryEntry;

pub(crate) fn assert_registry_entries(
    entries: &[NpmRegistryEntry],
    expected: &[(Option<&str>, &str)],
) {
    assert_eq!(entries.len(), expected.len());
    for (entry, (scope, url)) in entries.iter().zip(expected) {
        assert_eq!(entry.scope.as_deref(), *scope);
        assert_eq!(entry.url, *url);
    }
}
