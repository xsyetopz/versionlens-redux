use marked_yaml::types::MarkedScalarNode;

pub(super) struct PubspecDependencySource<'a> {
    pub(super) text: &'a str,
    pub(super) group: &'a str,
    pub(super) key: &'a MarkedScalarNode,
}
