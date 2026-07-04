mod alias;
mod ranked;
mod suffix;
mod versioned;

pub(super) use alias::latest_alias_tag;
pub(super) use suffix::latest_matching_suffix;
pub(super) use versioned::latest_versioned_tag;

pub(super) type DockerTagEntry<'a> = (&'a str, Option<&'a str>);
