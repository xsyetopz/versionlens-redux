use jsonc_parser::ast::{Object, Value};

pub(super) fn object_at_path<'a>(root: &'a Object<'a>, path: &[&str]) -> Option<&'a Object<'a>> {
    let Value::Object(object) = value_at_path(root, path)? else {
        return None;
    };
    Some(object)
}

pub(super) fn value_at_path<'a>(root: &'a Object<'a>, path: &[&str]) -> Option<&'a Value<'a>> {
    let mut current = root;
    let Some((last, parents)) = path.split_last() else {
        return None;
    };

    for segment in parents {
        current = current.get_object(segment)?;
    }
    current.get(last).map(|prop| &prop.value)
}
